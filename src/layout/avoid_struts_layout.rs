extern crate num_traits;
extern crate wtftw;

use self::num_traits::bounds::Bounded;
use std::collections::BTreeSet;
use self::wtftw::config::GeneralConfig;
use self::wtftw::core::stack::Stack;
use self::wtftw::layout::Direction;
use self::wtftw::layout::Layout;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::window_system::Rectangle;
use self::wtftw::window_system::Window;
use self::wtftw::window_system::WindowSystem;

#[derive(Clone, Copy)]
pub struct Strut(Direction, u64, u64, u64);

fn parse_strut_partial(x: Vec<u64>) -> Vec<Strut> {
    if x.len() != 12 {
        return Vec::new();
    }

    (vec!(Strut(Direction::Left, x[0], x[4], x[5]),
          Strut(Direction::Right, x[1], x[6], x[7]),
          Strut(Direction::Up, x[2], x[8], x[9]),
          Strut(Direction::Down, x[3], x[10], x[11]))).into_iter()
        .filter(|&Strut(_, n, _, _)| n != 0)
        .collect::<Vec<Strut>>()
    //match &x[..] {
        //[l, r, t, b, ly1, ly2, ry1, ry2, tx1, tx2, bx1, bx2] => {
            //(vec!(Strut(Direction::Left, l, ly1, ly2),
                  //Strut(Direction::Right, r, ry1, ry2),
                  //Strut(Direction::Up, t, tx1, tx2),
                  //Strut(Direction::Down, b, bx1, bx2))).into_iter()
                //.filter(|&Strut(_, n, _, _)| n != 0)
                //.collect()
        //},
        //_ => Vec::new()
    //}
}

pub fn get_strut(window_system: &WindowSystem, window: Window) -> Vec<Strut> {
    let partial_strut = window_system.get_partial_strut(window);

    fn parse_strut(x: Vec<u64>) -> Vec<Strut> {
        if x.len() != 4 {
            return Vec::new();
        }

        let s = vec!(Bounded::min_value(), Bounded::max_value());
        let r : Vec<u64> = x.iter().chain(s.iter().cycle()).take(12).map(|&x| x).collect();
        parse_strut_partial(r)

        //match &x[..] {
            //[a, b, c, d] => {
                //let t = vec!(a, b, c, d);
                //let s = vec!(Bounded::min_value(), Bounded::max_value());
                //let r : Vec<u64> = t.iter().chain(s.iter().cycle()).take(12).map(|&x| x).collect();
                //parse_strut_partial(r)
            //}
            //_ => Vec::new()
        //}
    }

    match partial_strut {
        Some(ps) => parse_strut_partial(ps),
        None     => {
            let strut = window_system.get_strut(window);
            match strut {
                Some(s) => parse_strut(s),
                None    => Vec::new()
            }
        }
    }
}

/// A layout that avoids dock like windows (e.g. dzen, xmobar, ...)
/// to not overlap them.
pub struct AvoidStrutsLayout {
    directions: BTreeSet<Direction>,
    layout: Box<Layout>
}

impl AvoidStrutsLayout {
    /// Create a new AvoidStrutsLayout, containing the given layout
    /// and avoiding struts in the given directions.
    pub fn new(d: Vec<Direction>, layout: Box<Layout>) -> Box<Layout> {
        Box::new(AvoidStrutsLayout {
            directions: d.iter().map(|&x| x).collect(),
            layout: layout.copy()
        })
    }
}

impl Layout for AvoidStrutsLayout {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {

        let new_screen = stack.clone().map_or(screen, |_| {
            window_system.get_windows().into_iter()
                .filter(|&w| window_system.is_dock(w) &&
                        window_system.get_geometry(w).overlaps(&screen))
                .flat_map(|x| get_strut(window_system, x).into_iter())
                .filter(|&Strut(s, _, _, _)| self.directions.contains(&s))
                .fold(screen, |Rectangle(x, y, w, h), Strut(d, sw, _, _)| {
                    let s = sw as u32;
                    match d {
                        Direction::Up    => Rectangle(x, y + s as i32, w, h - s),
                        Direction::Down  => Rectangle(x, y, w, h - s),
                        Direction::Left  => Rectangle(x + s as i32, y, w - s, h),
                        Direction::Right => Rectangle(x, y, w - s, h)
                    }
                })
        });

        self.layout.apply_layout(window_system, new_screen, config, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(AvoidStrutsLayout {
            directions: self.directions.clone(),
            layout: self.layout.copy()
        })
    }
}
