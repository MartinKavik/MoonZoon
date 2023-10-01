use crate::*;
use std::mem;

pub trait RawElWrapper {
    type RawEl: RawEl;

    fn raw_el_mut(&mut self) -> &mut Self::RawEl;

    #[track_caller]
    fn update_raw_el(mut self, updater: impl FnOnce(Self::RawEl) -> Self::RawEl) -> Self where Self: Sized {
        let raw_el = mem::replace(self.raw_el_mut(), RawEl::new_dummy());
        mem::swap(self.raw_el_mut(), &mut updater(raw_el));
        self
    }

    fn into_raw_el(mut self) -> Self::RawEl where Self: Sized {
        mem::replace(self.raw_el_mut(), RawEl::new_dummy())
    }
}
