use educe::Educe;
use std::sync::Arc;
use zoon::*;

#[derive(Educe)]
#[educe(Default(new))]
struct Store {
    #[educe(Default(expression = r#"Mutable::new(Arc::new("Hello".to_owned()))"#))]
    text_a: Mutable<Arc<String>>,
    #[educe(Default(expression = r#"Mutable::new(Arc::new("World!".to_owned()))"#))]
    text_b: Mutable<Arc<String>>,
    joined_texts: Mutable<Arc<String>>,
}

#[static_ref]
fn store() -> &'static Store {
    Store::new()
}

fn main() {
    // @TODO / WARNING: experimental
    Task::start_blocking(|_scope| {
        map_ref! {
            let text_a = store().text_a.signal_cloned(),
            let text_b = store().text_b.signal_cloned() => {
                let joined_texts = format!("{text_a} {text_b}");
                store().joined_texts.set(joined_texts.into());
            }
        }
        .to_future()
    });
    start_app("app", root);
}

pub fn root() -> impl Element {
    Column::new()
        .s(Padding::all(20).top(150))
        .s(Align::new().center_x())
        .s(Gap::new().y(70))
        .item(field("Text A", store().text_a.clone(), false))
        .item(field("Text B", store().text_b.clone(), false))
        .item(field("Joined texts", store().joined_texts.clone(), true))
}

fn field(label: &str, text: Mutable<Arc<String>>, is_output: bool) -> impl Element {
    Column::new()
        .s(Gap::new().y(15))
        .item(Label::new().for_input(label).label(label))
        .item(
            TextArea::new()
                .id(label)
                .s(Width::exact(350))
                .s(Height::exact(if is_output { 160 } else { 80 }))
                .s(Align::new().center_x())
                .s(Outline::outer())
                .s(Padding::new().x(4).y(2))
                .s(Cursor::new(is_output.then(|| CursorIcon::Default)))
                .s(Background::new().color(is_output.then_some(hsluv!(0, 0, 95))))
                .s(Resizable::y())
                .read_only(is_output)
                .text_signal(text.signal_cloned())
                .on_change(move |new_text| text.set(new_text.into())),
        )
}
