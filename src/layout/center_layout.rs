extern crate wtftw;

use std::borrow::ToOwned;
use self::wtftw::core::stack::Stack;
use self::wtftw::config::GeneralConfig;
use self::wtftw::window_system::Rectangle;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::layout::Layout;
use self::wtftw::window_system::WindowSystem;
use self::wtftw::window_system::Window;


pub struct CenterLayout {
    pub layout: Box<Layout>
}

impl CenterLayout {
    pub fn new(layout: Box<Layout>) -> Box<Layout> {
        Box::new(CenterLayout {
            layout: layout.copy()
        })
    }
}

impl Layout for CenterLayout  {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                if s.len() == 1 {
                    self.layout.apply_layout(window_system, screen, config, &Some(s.clone()))
                } else {
                    let new_stack = if s.up.len() > 0 {
                        Stack::<Window>::new(s.up[0],
                                             s.up.iter().skip(1).map(|&x| x).collect(),
                                             s.down.clone())
                    } else {
                        Stack::<Window>::new(s.down[0],
                                             Vec::new(),
                                             s.down.iter().skip(1).map(|&x| x).collect())
                    };
                    (vec!({
                        let x = screen.0 + ((screen.2 as f32 * 0.2) as i32 / 2);
                        let y = screen.1 + ((screen.3 as f32 * 0.2) as i32 / 2);
                        let w = (screen.2 as f32 * 0.8) as u32;
                        let h = (screen.3 as f32 * 0.8) as u32;
                        (s.focus, Rectangle(x, y, w, h))
                    }).into_iter()).chain(self.layout.apply_layout(window_system, screen, config,
                                                                   &Some(new_stack)).into_iter()).collect()
                }
            },
            _ => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        "Center".to_owned()
    }

    fn copy(&self) -> Box<Layout> {
        CenterLayout::new(self.layout.copy())
    }
}
