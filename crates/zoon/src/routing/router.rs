use crate::{routing::decode_uri_component, *};
use std::ops::Deref;
use web_sys::MouseEvent;

type UrlChangeSender = Sender<Option<Vec<String>>>;

#[derive(Clone, Default, Debug)]
pub enum RouteState<R: Clone> {
    #[default]
    NoRoute,
    UnknownRoute,
    KnownRoute(R),
}

pub struct Router<R: FromRouteSegments + IntoCowStr<'static> + Clone + 'static> {
    popstate_listener: SendWrapper<Closure<dyn Fn()>>,
    link_interceptor: SendWrapper<Closure<dyn Fn(MouseEvent)>>,
    url_change_sender: UrlChangeSender,
    current_route: Mutable<RouteState<R>>,
    previous_route: Mutable<RouteState<R>>,
    _url_change_handle: TaskHandle,
}

impl<R: FromRouteSegments + IntoCowStr<'static> + Clone + 'static> Default for Router<R> {
    fn default() -> Self {
        Self::new(|_| async {})
    }
}

impl<R: FromRouteSegments + IntoCowStr<'static> + Clone + 'static> Router<R> {
    pub fn new<O: Future<Output = ()> + 'static>(
        mut on_route_change: impl FnMut(Option<R>) -> O + 'static,
    ) -> Self {
        let current_route = Mutable::new(RouteState::default());
        let previous_route = Mutable::new(RouteState::default());
        let on_route_change = {
            let current_route = current_route.clone();
            let previous_route = previous_route.clone();
            move |route: Option<R>| {
                let old_current_route = current_route.replace(match route.clone() {
                    Some(route) => RouteState::KnownRoute(route),
                    None => RouteState::UnknownRoute,
                });
                previous_route.set(old_current_route);
                on_route_change(route)
            }
        };
        let (url_change_sender, _url_change_handle) = setup_url_change_handler(on_route_change);
        Router {
            popstate_listener: setup_popstate_listener(url_change_sender.clone()),
            link_interceptor: setup_link_interceptor(url_change_sender.clone()),
            url_change_sender,
            current_route,
            previous_route,
            _url_change_handle,
        }
    }

    pub fn go<'a>(&self, to: impl IntoCowStr<'a>) {
        go(&self.url_change_sender, to, false);
    }

    pub fn replace<'a>(&self, with: impl IntoCowStr<'a>) {
        replace(&self.url_change_sender, with, false);
    }

    pub fn silent_go<'a>(&self, to: impl IntoCowStr<'a>) {
        go(&self.url_change_sender, to, true);
    }

    pub fn silent_replace<'a>(&self, with: impl IntoCowStr<'a>) {
        replace(&self.url_change_sender, with, true);
    }

    pub fn route(&self) -> ReadOnlyMutable<RouteState<R>> {
        self.current_route.read_only()
    }

    pub fn previous_route(&self) -> ReadOnlyMutable<RouteState<R>> {
        self.previous_route.read_only()
    }

    pub fn go_to_previous_known_or_else<'a, ICS: IntoCowStr<'a>>(&self, to: impl FnOnce() -> ICS) {
        let previous_lock = self.previous_route.lock_ref();
        if let KnownRoute(route) = previous_lock.deref() {
            let route = route.clone();
            drop(previous_lock);
            self.go(route);
        } else {
            drop(previous_lock);
            self.go(to());
        }
    }
}

impl<R: FromRouteSegments + IntoCowStr<'static> + Clone + 'static> Drop for Router<R> {
    fn drop(&mut self) {
        window()
            .remove_event_listener_with_callback(
                "popstate",
                self.popstate_listener.as_ref().unchecked_ref(),
            )
            .unwrap_throw();

        document()
            .remove_event_listener_with_callback(
                "click",
                self.link_interceptor.as_ref().unchecked_ref(),
            )
            .unwrap_throw();
    }
}

// ------ helpers -------

fn setup_url_change_handler<R: FromRouteSegments, O: Future<Output = ()> + 'static>(
    mut on_route_change: impl FnMut(Option<R>) -> O + 'static,
) -> (UrlChangeSender, TaskHandle) {
    let (url_change_sender, url_change_receiver) = channel(current_url_segments());
    let url_change_handler = url_change_receiver.for_each(move |segments| {
        let route = segments.and_then(R::from_route_segments);
        on_route_change(route)
    });
    let url_change_handle = Task::start_droppable(url_change_handler);
    (url_change_sender, url_change_handle)
}

fn go<'a>(url_change_sender: &UrlChangeSender, to: impl IntoCowStr<'a>, silent: bool) {
    let to = to.into_cow_str();
    if !to.starts_with('/') {
        return window().location().assign(&to).unwrap_throw();
    }
    history()
        .push_state_with_url(&JsValue::NULL, "", Some(&to))
        .unwrap_throw();
    if !silent {
        url_change_sender
            .send(current_url_segments())
            .unwrap_throw();
    }
}

fn replace<'a>(url_change_sender: &UrlChangeSender, with: impl IntoCowStr<'a>, silent: bool) {
    let with = with.into_cow_str();
    if !with.starts_with('/') {
        return window().location().replace(&with).unwrap_throw();
    }
    history()
        .replace_state_with_url(&JsValue::NULL, "", Some(&with))
        .unwrap_throw();
    if !silent {
        url_change_sender
            .send(current_url_segments())
            .unwrap_throw();
    }
}

fn current_url_segments() -> Option<Vec<String>> {
    let path = window().location().pathname().unwrap_throw();
    let mut segments = Vec::new();
    for segment in path.trim_start_matches('/').split_terminator('/') {
        match decode_uri_component(segment) {
            Ok(segment) => segments.push(segment),
            Err(error) => {
                crate::eprintln!(
                    "Cannot decode the URL segment '{}'. Error: {:#?}",
                    segment,
                    error
                );
                None?
            }
        }
    }
    Some(segments)
}

fn setup_popstate_listener(url_change_sender: UrlChangeSender) -> SendWrapper<Closure<dyn Fn()>> {
    let closure = Closure::new(move || {
        url_change_sender
            .send(current_url_segments())
            .unwrap_throw();
    });

    window()
        .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
        .unwrap_throw();

    SendWrapper::new(closure)
}

fn setup_link_interceptor(
    url_change_sender: UrlChangeSender,
) -> SendWrapper<Closure<dyn Fn(MouseEvent)>> {
    let closure = Closure::new(move |event| {
        link_click_handler(event, &url_change_sender);
    });

    document()
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap_throw();

    SendWrapper::new(closure)
}

fn link_click_handler(event: MouseEvent, url_change_sender: &UrlChangeSender) -> Option<()> {
    if event.ctrl_key() || event.meta_key() || event.shift_key() || event.button() != 0 {
        None?
    }
    let ws_element: web_sys::Element = event.target()?.dyn_into().ok()?;
    let a: web_sys::Element = ws_element
        .closest(r#"a[href^="/"]:not([download], [target="_blank"])"#)
        .ok()??;
    let href = a.get_attribute("href")?;
    event.prevent_default();
    go(url_change_sender, href, false);
    Some(())
}
