use crate::style::{px, DynamicCSSProps, StaticCSSProps};
use crate::*;

#[derive(Default)]
pub struct Height<'a> {
    static_css_props: StaticCSSProps<'a>,
    dynamic_css_props: DynamicCSSProps,
}

impl<'a> Height<'a> {
    pub fn new(height: u32) -> Self {
        let mut this = Self::default();
        this.static_css_props.insert("height", px(height));
        this
    }
}

impl<'a> Style<'a> for Height<'a> {
    fn into_css_props(self) -> (StaticCSSProps<'a>, DynamicCSSProps) {
        (self.static_css_props, self.dynamic_css_props)
    }
}