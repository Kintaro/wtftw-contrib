extern crate wtftw;

use self::wtftw::config::GeneralConfig;
use self::wtftw::core::stack::Stack;
use self::wtftw::layout::Layout;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::window_system::Rectangle;
use self::wtftw::window_system::Window;
use self::wtftw::window_system::WindowSystem;

pub struct LayoutCollection {
    pub layouts: Vec<Box<dyn Layout>>,
    pub current: usize,
}

impl LayoutCollection {
    pub fn boxed_new(layouts: Vec<Box<dyn Layout>>) -> Box<dyn Layout> {
        Box::new(LayoutCollection {
            layouts,
            current: 0,
        })
    }
}

impl Layout for LayoutCollection {
    fn apply_layout(
        &mut self,
        window_system: &dyn WindowSystem,
        screen: Rectangle,
        config: &GeneralConfig,
        stack: &Option<Stack<Window>>,
    ) -> Vec<(Window, Rectangle)> {
        self.layouts[self.current].apply_layout(window_system, screen, config, stack)
    }

    fn apply_message(
        &mut self,
        message: LayoutMessage,
        window_system: &dyn WindowSystem,
        stack: &Option<Stack<Window>>,
        config: &GeneralConfig,
    ) -> bool {
        match message {
            LayoutMessage::Next => {
                self.layouts[self.current].unhook(window_system, stack, config);
                self.current = (self.current + 1) % self.layouts.len();
                true
            }
            LayoutMessage::Prev => {
                self.layouts[self.current].unhook(window_system, stack, config);
                self.current = (self.current + (self.layouts.len() - 1)) % self.layouts.len();
                true
            }
            _ => self.layouts[self.current].apply_message(message, window_system, stack, config),
        }
    }

    fn description(&self) -> String {
        self.layouts[self.current].description()
    }

    fn copy(&self) -> Box<dyn Layout> {
        Box::new(LayoutCollection {
            current: self.current,
            layouts: self.layouts.iter().map(|x| x.copy()).collect(),
        })
    }
}
