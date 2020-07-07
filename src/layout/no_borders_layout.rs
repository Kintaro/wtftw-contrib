extern crate wtftw;

use self::wtftw::layout::Layout;
use super::with_borders_layout::WithBordersLayout;

pub struct NoBordersLayout;

impl NoBordersLayout {
    pub fn boxed_new(layout: Box<dyn Layout>) -> Box<dyn Layout> {
        WithBordersLayout::boxed_new(0, layout)
    }
}
