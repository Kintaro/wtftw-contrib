extern crate wtftw;

use self::wtftw::config::GeneralConfig;
use self::wtftw::core::stack::Stack;
use self::wtftw::layout::Layout;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::window_system::Rectangle;
use self::wtftw::window_system::Window;
use self::wtftw::window_system::WindowSystem;

pub struct WithBordersLayout {
    border: u32,
    layout: Box<Layout>
}

impl WithBordersLayout {
    pub fn new(border: u32, layout: Box<Layout>) -> Box<Layout> {
        Box::new(WithBordersLayout {
            border: border,
            layout: layout.copy()
        })
    }
}

impl Layout for WithBordersLayout {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, self.border);
            }
        }
        self.layout.apply_layout(window_system, screen, config, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(WithBordersLayout {
            border: self.border,
            layout: self.layout.copy()
        })
    }

    fn unhook(&self, window_system: &WindowSystem, stack: &Option<Stack<Window>>, config: &GeneralConfig) {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, config.border_width);
                let Rectangle(_, _, w, h) = window_system.get_geometry(window);
                window_system.resize_window(window, w + 2 * config.border_width, h + 2 * config.border_width);
            }
        }
    }
}
