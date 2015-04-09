 extern crate wtftw;
 
use std::iter;
use std::borrow::ToOwned;
use self::wtftw::core::stack::Stack;
use self::wtftw::config::GeneralConfig;
use self::wtftw::window_system::Rectangle;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::layout::Layout;
use self::wtftw::window_system::WindowSystem;
use self::wtftw::window_system::Window;
use self::wtftw::window_manager::ScreenDetail;

#[derive(Clone)]
pub struct ResizableTallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32,
    pub slaves: Vec<f32>
}

impl ResizableTallLayout {
    pub fn new() -> Box<Layout> {
        Box::new(ResizableTallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5,
            slaves: Vec::new()
        })
    }

    fn tile<U>(ratio: f32, mf: U, screen: ScreenDetail, num_master: u32, num_windows: u32) -> Vec<Rectangle> 
where
            U : Iterator<Item=f32> {
        if num_windows <= num_master || num_master == 0 {
            ResizableTallLayout::split_vertically(mf, num_windows, screen)
        } else {
            let v = mf.collect::<Vec<_>>();
            let (r1, r2) = ResizableTallLayout::split_horizontally_by(ratio, screen);
            let v1 = ResizableTallLayout::split_vertically(v.clone().into_iter(), num_master, r1);
            let v2 = ResizableTallLayout::split_vertically(v.clone().into_iter().skip(num_master as usize), 
num_windows - num_master, r2);
            v1.iter().chain(v2.iter()).map(|&x| x).collect()
        }
    }

    fn split_vertically<U>(r: U, num: u32, screen: ScreenDetail) -> Vec<Rectangle> where
            U : Iterator<Item=f32> {
        if r.size_hint().0 == 0 {
            return vec!(screen);
        }

        if num < 2 {
            return vec!(screen);
        }

        let Rectangle(sx, sy, sw, sh) = screen;
        let fxv = r.collect::<Vec<_>>();
        let f = fxv[0];
        let smallh = ((sh / num) as f32 * f) as u32;
        (vec!(Rectangle(sx, sy, sw, smallh))).iter()
            .chain(ResizableTallLayout::split_vertically(fxv.into_iter().skip(1), num - 1,
                                                         Rectangle(sx, sy + smallh as i32, sw, sh - 
smallh)).iter())
            .map(|&x| x)
            .collect()
    }

    fn split_horizontally_by(ratio: f32, screen: ScreenDetail) -> (Rectangle, Rectangle) {
        let Rectangle(sx, sy, sw, sh) = screen;
        let leftw = (sw as f32 * ratio).floor() as u32;

        (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw as i32, sy, sw - leftw, sh))
    }

    fn resize(&mut self, stack: &Option<Stack<Window>>, d: f32) {
        fn modify<U>(v: U, d: f32, n: usize) -> Vec<f32> where U : Iterator<Item=f32> {
            if v.size_hint().0 == 0 { return Vec::new(); }
            if n == 0 {
                let frac = v.collect::<Vec<_>>();
                (vec!(frac[0] + d)).into_iter().chain(frac.into_iter().skip(1)).collect()
            } else {
                let frac = v.collect::<Vec<_>>();
                (vec!(frac[0])).into_iter()
                    .chain(modify(frac.into_iter().skip(1), d, n - 1).into_iter())
                    .collect()
            }
        }

        if let &Some(ref s) = stack {
            let n = s.up.len();
            let total = s.len();
            let pos = if n as u32 == self.num_master - 1 || n == total - 1 { n - 1 } else { n };
            let mfrac = modify(self.slaves.clone().into_iter().chain(iter::repeat(1.0)).take(total), d, pos);
            self.slaves = mfrac.into_iter().take(total).collect();
        }
    }
}

impl Layout for ResizableTallLayout {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, _: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                let ws = s.integrate();
                s.integrate().iter()
                    .zip(ResizableTallLayout::tile(self.ratio,
                                                   
self.slaves.clone().into_iter().chain(iter::repeat(1.0)).take(ws.len()),
                                                   screen, self.num_master, ws.len() as u32).iter())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage, _: &WindowSystem,
                         stack: &Option<Stack<Window>>, _: &GeneralConfig) -> bool {
        let d = self.increment_ratio;
        match message {
            LayoutMessage::Increase => { self.ratio += self.increment_ratio; true }
            LayoutMessage::Decrease => { self.ratio -= self.increment_ratio; true }
            LayoutMessage::IncreaseMaster => { self.num_master += 1; true }
            LayoutMessage::DecreaseMaster => {
                if self.num_master > 1 { self.num_master -= 1 } true
            }
            LayoutMessage::IncreaseSlave => { self.resize(stack,  d); true }
            LayoutMessage::DecreaseSlave => { self.resize(stack, -d); true }
            _                       => false
        }
    }

    fn description(&self) -> String {
        "ResizeTall".to_owned()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}
