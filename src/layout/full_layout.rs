extern crate wtftw;

use self::wtftw::config::GeneralConfig;
use self::wtftw::core::stack::Stack;
use self::wtftw::layout::Layout;
use self::wtftw::window_system::Rectangle;
use self::wtftw::window_system::Window;
use self::wtftw::window_system::WindowSystem;
use std::borrow::ToOwned;

#[derive(Copy, Clone)]
pub struct FullLayout;

impl Layout for FullLayout {
    fn apply_layout(
        &mut self,
        _: &dyn WindowSystem,
        screen: Rectangle,
        config: &GeneralConfig,
        stack: &Option<Stack<Window>>,
    ) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref st) => {
                let bw = 2 * config.border_width;
                let Rectangle(x, y, sw, sh) = screen;
                vec![(st.focus, Rectangle(x, y, sw + bw, sh + bw))]
            }
            None => Vec::new(),
        }
    }

    fn description(&self) -> String {
        "Full".to_owned()
    }

    fn copy(&self) -> Box<dyn Layout> {
        Box::new(*self)
    }
}
