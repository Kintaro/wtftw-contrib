extern crate wtftw;

use self::wtftw::config::GeneralConfig;
use self::wtftw::core::stack::Stack;
use self::wtftw::layout::Layout;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::window_system::Rectangle;
use self::wtftw::window_system::Window;
use self::wtftw::window_system::WindowSystem;
use std::borrow::ToOwned;

pub struct CenterLayout {
    pub layout: Box<dyn Layout>,
}

impl CenterLayout {
    pub fn boxed_new(layout: Box<dyn Layout>) -> Box<dyn Layout> {
        Box::new(CenterLayout {
            layout: layout.copy(),
        })
    }
}

impl Layout for CenterLayout {
    fn apply_layout(
        &mut self,
        window_system: &dyn WindowSystem,
        screen: Rectangle,
        config: &GeneralConfig,
        stack: &Option<Stack<Window>>,
    ) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref stack) => {
                if stack.len() == 1 {
                    self.layout
                        .apply_layout(window_system, screen, config, &Some(stack.clone()))
                } else {
                    let new_stack = if !stack.up.is_empty() {
                        Stack::<Window>::new(
                            stack.up[0],
                            stack.up.iter().skip(1).copied().collect(),
                            stack.down.clone(),
                        )
                    } else {
                        Stack::<Window>::new(
                            stack.down[0],
                            Vec::new(),
                            stack.down.iter().skip(1).copied().collect(),
                        )
                    };
                    (vec![{
                        let x = screen.0 + ((screen.2 as f32 * 0.2) as i32 / 2);
                        let y = screen.1 + ((screen.3 as f32 * 0.2) as i32 / 2);
                        let w = (screen.2 as f32 * 0.8) as u32;
                        let h = (screen.3 as f32 * 0.8) as u32;
                        (stack.focus, Rectangle(x, y, w, h))
                    }]
                    .into_iter())
                    .chain(
                        self.layout
                            .apply_layout(window_system, screen, config, &Some(new_stack))
                            .into_iter(),
                    )
                    .collect()
                }
            }
            _ => Vec::new(),
        }
    }

    fn apply_message(
        &mut self,
        message: LayoutMessage,
        window_system: &dyn WindowSystem,
        stack: &Option<Stack<Window>>,
        config: &GeneralConfig,
    ) -> bool {
        self.layout
            .apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        "Center".to_owned()
    }

    fn copy(&self) -> Box<dyn Layout> {
        CenterLayout::boxed_new(self.layout.copy())
    }
}
