use crate::*;
use std::marker::PhantomData;

// ------ ------
//   Element
// ------ ------

make_flags!(Child);

pub struct El<ChildFlag> {
    raw_el: RawHtmlEl,
    flags: PhantomData<ChildFlag>,
}

impl El<ChildFlagNotSet> {
    pub fn new() -> Self {
        Self {
            raw_el: RawHtmlEl::new("div").attr("class", "el"),
            flags: PhantomData,
        }
    }
}

impl Element for El<ChildFlagSet> {
    fn into_raw_element(self) -> RawElement {
        self.raw_el.into()
    }
}

// ------ ------
//  Attributes
// ------ ------

impl<'a, ChildFlag> El<ChildFlag> {
    pub fn child(mut self, child: impl IntoElement<'a> + 'a) -> El<ChildFlagSet>
    where
        ChildFlag: FlagNotSet,
    {
        self.raw_el = self.raw_el.child(child);
        self.into_type()
    }

    pub fn child_signal(
        mut self,
        child: impl Signal<Item = impl IntoElement<'a>> + Unpin + 'static,
    ) -> El<ChildFlagSet>
    where
        ChildFlag: FlagNotSet,
    {
        self.raw_el = self.raw_el.child_signal(child);
        self.into_type()
    }

    fn into_type<NewChildFlag>(self) -> El<NewChildFlag> {
        El {
            raw_el: self.raw_el,
            flags: PhantomData,
        }
    }
}
