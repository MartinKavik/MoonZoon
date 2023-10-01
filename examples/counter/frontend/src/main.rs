use zoon::*;

fn main() {
    start_app("app", root);
}

fn root() -> impl Element {
    let el = Box::new(El::new().child("Hello")) as Box<dyn Element>;

    // let x = el.into_raw();

    Stack::new()
        .layer(el)

    // Stack::new()
    //     .layer(x)

    // Text::new("dedgr")
}

