extern crate wtftw;

use std::ops::Deref;
use std::borrow::ToOwned;
use self::wtftw::core::stack::Stack;
use self::wtftw::config::GeneralConfig;
use self::wtftw::window_system::Rectangle;
use self::wtftw::layout::Direction;
use self::wtftw::layout::LayoutMessage;
use self::wtftw::layout::Layout;
use self::wtftw::window_system::WindowSystem;
use self::wtftw::window_system::Window;

#[derive(Clone, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical
}

impl Axis {
    pub fn opposite(&self) -> Axis {
        match self {
            &Axis::Horizontal => Axis::Vertical,
            &Axis::Vertical   => Axis::Horizontal
        }
    }
}

#[derive(Clone)]
pub enum Tree<T> {
    Leaf,
    Node(T, Box<Tree<T>>, Box<Tree<T>>)
}

impl<T> Tree<T> {
    pub fn number_of_leaves(&self) -> usize {
        match self {
            &Tree::Leaf => 1,
            &Tree::Node(_, ref l, ref r) => l.number_of_leaves() + r.number_of_leaves()
        }
    }
}

#[derive(Clone)]
pub struct Split {
    axis: Axis,
    ratio: f32
}

impl Split {
    pub fn new(axis: Axis, r: f32) -> Split {
        Split { axis: axis, ratio: r }
    }

    pub fn split(&self, Rectangle(x, y, w, h): Rectangle) -> (Rectangle, Rectangle) {
        match self.axis {
            Axis::Horizontal => {
                let hr = (h as f32 * self.ratio) as u32;
                (Rectangle(x, y, w, hr), Rectangle(x, y + hr as i32, w, h - hr))
            },
            Axis::Vertical => {
                let wr = (w as f32 * self.ratio) as u32;
                (Rectangle(x, y, wr, h), Rectangle(x + wr as i32, y, w - wr, h))
            }
        }
    }

    pub fn opposite(&self) -> Split {
        Split { axis: self.axis.opposite(), ratio: self.ratio }
    }

    pub fn increase_ratio(&self, r: f32) -> Split {
        Split { axis: self.axis.clone(), ratio: self.ratio + r }
    }
}

#[derive(Clone)]
pub enum Crumb<T> {
    LeftCrumb(T, Tree<T>),
    RightCrumb(T, Tree<T>)
}

impl<T: Clone> Crumb<T> {
    pub fn swap(&self) -> Crumb<T> {
        match self {
            &Crumb::LeftCrumb(ref s, ref t)  => Crumb::RightCrumb(s.clone(), t.clone()),
            &Crumb::RightCrumb(ref s, ref t) => Crumb::LeftCrumb(s.clone(), t.clone())
        }
    }

    pub fn parent(&self) -> T {
        match self {
            &Crumb::LeftCrumb(ref s, _)  => s.clone(),
            &Crumb::RightCrumb(ref s, _) => s.clone()
        }
    }

    pub fn modify_parent<F>(&self, f: F) -> Crumb<T> where F: Fn(&T) -> T {
        match self {
            &Crumb::LeftCrumb(ref s, ref t)  => Crumb::LeftCrumb(f(s), t.clone()),
            &Crumb::RightCrumb(ref s, ref t) => Crumb::RightCrumb(f(s), t.clone())
        }
    }
}

#[derive(Clone)]
pub struct Zipper {
    tree: Tree<Split>,
    crumbs: Vec<Crumb<Split>>
}

impl Zipper {
    fn left_append<S>(x: S, v: Vec<S>) -> Vec<S> {
        (vec!(x)).into_iter().chain(v.into_iter()).collect()
    }

    pub fn from_tree(tree: Tree<Split>) -> Zipper {
        Zipper {
            tree: tree.clone(),
            crumbs: Vec::new()
        }
    }

    pub fn go_left(&self) -> Option<Zipper> {
        match &self.tree {
            &Tree::Leaf => None,
            &Tree::Node(ref x, ref l, ref r) => Some(Zipper {
                tree: l.deref().clone(),
                crumbs: Zipper::left_append::<Crumb<Split>>(Crumb::LeftCrumb(x.clone(), r.deref().clone()), 
self.crumbs.clone())
            })
        }
    }

    pub fn go_right(&self) -> Option<Zipper> {
        match &self.tree {
            &Tree::Leaf => None,
            &Tree::Node(ref x, ref l, ref r) => Some(Zipper {
                tree: r.deref().clone(),
                crumbs: Zipper::left_append::<Crumb<Split>>(Crumb::RightCrumb(x.clone(), l.deref().clone()), 
self.crumbs.clone())
            })
        }
    }

    pub fn go_up(&self) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            None
        } else {
            let head = self.crumbs[0].clone();
            let rest = if self.crumbs.len() == 1 {
                Vec::new()
            } else {
                self.crumbs.clone().into_iter().skip(1).collect()
            };

            match head {
                Crumb::LeftCrumb(x, r)  => Some(Zipper { tree: Tree::Node(x, Box::new(self.tree.clone()),
                                                                          Box::new(r)), crumbs: rest }),
                Crumb::RightCrumb(x, l) => Some(Zipper { tree: Tree::Node(x, Box::new(l),
                                                                          Box::new(self.tree.clone())),
                    crumbs: rest })
            }
        }
    }

    pub fn go_sibling(&self) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return None;
        }

        let head = self.crumbs[0].clone();

        match head {
            Crumb::LeftCrumb(_, _) => self.go_up().and_then(|x| x.go_right()),
            Crumb::RightCrumb(_, _) => self.go_up().and_then(|x| x.go_left())
        }
    }

    pub fn go_to_nth_leaf(&self, n: usize) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => Some(self.clone()),
            Tree::Node(_, ref l, _)  => {
                if l.number_of_leaves() > n {
                    self.go_left().and_then(|x| x.go_to_nth_leaf(n))
                } else {
                    self.go_right().and_then(|x| x.go_to_nth_leaf(n - l.number_of_leaves()))
                }
            }
        }
    }

    pub fn split_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Node(Split::new(Axis::Vertical, 0.5),
                                                   Box::new(Tree::Leaf), Box::new(Tree::Leaf)),
                        crumbs: Vec::new() })
                } else {
                    let head = self.crumbs[0].clone();
                    Some(Zipper {
                        tree: Tree::Node(Split::new(head.parent().axis.opposite(), 0.5),
                                         Box::new(Tree::Leaf), Box::new(Tree::Leaf)),
                        crumbs: self.crumbs.clone()
                    })
                }
            }
            _ => None
        }
    }

    pub fn remove_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    None
                } else {
                    let head = self.crumbs[0].clone();
                    let rest = if self.crumbs.len() == 1 {
                        Vec::new()
                    } else {
                        self.crumbs.clone().into_iter().skip(1).collect()
                    };
                    match head {
                        Crumb::LeftCrumb(_, r) => Some(Zipper { tree: r.clone(), crumbs: rest }),
                        Crumb::RightCrumb(_, l) => Some(Zipper { tree: l.clone(), crumbs: rest })
                    }
                }
            },
            _ => None
        }
    }

    pub fn rotate_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Leaf, crumbs: Vec::new() })
                } else {
                    let mut c = self.crumbs.clone();
                    c[0] = c[0].modify_parent(|x| x.opposite());
                    Some(Zipper {
                        tree: Tree::Leaf,
                        crumbs: c
                    })
                }
            }
            _ => None
        }
    }

    pub fn swap_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Leaf, crumbs: Vec::new() })
                } else {
                    let mut c = self.crumbs.clone();
                    c[0] = c[0].swap();
                    Some(Zipper {
                        tree: Tree::Leaf,
                        crumbs: c
                    })
                }
            }
            _ => None
        }
    }

    pub fn is_all_the_way(&self, dir: Direction) -> bool {
        if self.crumbs.is_empty() {
            return true;
        }

        let head = self.crumbs[0].clone();
        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Vertical   => false,
            (Direction::Left,  Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Vertical   => false,
            (Direction::Down,  Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Horizontal => false,
            (Direction::Up,    Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Horizontal => false,
            _ => self.go_up().map_or(false, |x| x.is_all_the_way(dir))
        }
    }

    pub fn expand_towards(&self, dir: Direction) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return Some(self.clone());
        }

        if self.is_all_the_way(dir) {
            return None;
        }

        let head = self.crumbs[0].clone();
        let rest = if self.crumbs.len() == 1 {
            Vec::new()
        } else {
            self.crumbs.clone().into_iter().skip(1).collect()
        };

        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, ref r)) if s.axis == Axis::Vertical => Some(Zipper {
                tree: self.tree.clone(),
                crumbs: Zipper::left_append(Crumb::LeftCrumb(s.increase_ratio(0.05), r.clone()), rest)
            }),
            (Direction::Left, Crumb::RightCrumb(ref s, ref r)) if s.axis == Axis::Vertical => Some(Zipper {
                tree: self.tree.clone(),
                crumbs: Zipper::left_append(Crumb::RightCrumb(s.increase_ratio(-0.05), r.clone()), rest)
            }),
            (Direction::Down, Crumb::LeftCrumb(ref s, ref r)) if s.axis == Axis::Horizontal => Some(Zipper {
                tree: self.tree.clone(),
                crumbs: Zipper::left_append(Crumb::LeftCrumb(s.increase_ratio(0.05), r.clone()), rest)
            }),
            (Direction::Up, Crumb::RightCrumb(ref s, ref r)) if s.axis == Axis::Horizontal => Some(Zipper {
                tree: self.tree.clone(),
                crumbs: Zipper::left_append(Crumb::RightCrumb(s.increase_ratio(-0.05), r.clone()), rest)
            }),
            _ => self.go_up().and_then(|x| x.expand_towards(dir))
        }
    }

    pub fn shrink_from(&self, dir: Direction) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return Some(self.clone());
        }

        let head = self.crumbs[0].clone();

        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Vertical   => 
self.go_sibling().and_then(|x| x.expand_towards(Direction::Left)),
            (Direction::Left,  Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Vertical   => 
self.go_sibling().and_then(|x| x.expand_towards(Direction::Right)),
            (Direction::Down,  Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Horizontal => 
self.go_sibling().and_then(|x| x.expand_towards(Direction::Up)),
            (Direction::Up,    Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Horizontal => 
self.go_sibling().and_then(|x| x.expand_towards(Direction::Down)),
            _ => self.go_up().and_then(|x| x.shrink_from(dir))
        }
    }

    pub fn top(&self) -> Zipper {
        self.go_up().map_or(self.clone(), |x| x.top())
    }

    pub fn to_tree(&self) -> Tree<Split> {
        self.top().tree.clone()
    }
}

#[derive(Clone)]
pub struct BinarySpacePartition {
    tree: Option<Tree<Split>>
}

impl BinarySpacePartition {
    pub fn new() -> Box<Layout> {
        Box::new(BinarySpacePartition::empty())
    }

    pub fn empty() -> BinarySpacePartition {
        BinarySpacePartition { tree: None }
    }

    pub fn make(tree: Tree<Split>) -> BinarySpacePartition {
        BinarySpacePartition { tree: Some(tree) }
    }

    pub fn make_zipper(&self) -> Option<Zipper> {
        self.tree.clone().map(|x| Zipper::from_tree(x))
    }

    pub fn size(&self) -> usize {
        self.tree.clone().map_or(0, |x| x.number_of_leaves())
    }

    pub fn from_zipper(zipper: Option<Zipper>) -> BinarySpacePartition {
        BinarySpacePartition {
            tree: zipper.clone().map(|x| x.top().to_tree())
        }
    }

    pub fn rectangles(&self, rect: Rectangle) -> Vec<Rectangle> {
        self.tree.clone().map_or(Vec::new(), |t| {
            match t {
                Tree::Leaf => vec!(rect),
                Tree::Node(value, l, r) => {
                    let (left_box, right_box) = value.split(rect);
                    let left  = BinarySpacePartition::make(l.deref().clone()).rectangles(left_box);
                    let right = BinarySpacePartition::make(r.deref().clone()).rectangles(right_box);
                    left.into_iter().chain(right.into_iter()).collect()
                }
            }
        })
    }

    pub fn do_to_nth<F>(&self, n: usize, f: F) -> BinarySpacePartition where F: Fn(Zipper) -> Option<Zipper> {
        BinarySpacePartition::from_zipper(self.make_zipper().and_then(|x| x.go_to_nth_leaf(n)).and_then(f))
    }

    pub fn split_nth(&self, n: usize) -> BinarySpacePartition {
        if self.tree.is_none() {
            BinarySpacePartition::make(Tree::Leaf)
        } else {
            self.do_to_nth(n, |x| x.split_current_leaf())
        }
    }

    pub fn remove_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => BinarySpacePartition::empty(),
                    _           => self.do_to_nth(n, |x| x.remove_current_leaf())
                }
            }
        }
    }

    pub fn rotate_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.rotate_current_leaf())
                }
            }
        }
    }

    pub fn swap_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.swap_current_leaf())
                }
            }
        }
    }

    pub fn grow_nth_towards(&self, dir: Direction, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.expand_towards(dir))
                }
            }
        }
    }

    pub fn shrink_nth_from(&self, dir: Direction, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.shrink_from(dir))
                }
            }
        }

    }

    fn to_index<T: Clone + Eq>(s: Option<Stack<T>>) -> (Vec<T>, Option<usize>) {
        match s {
            None => (Vec::new(), None),
            Some(x) => (x.integrate(), Some(x.up.len()))
        }
    }

    fn stack_index<T: Clone + Eq>(s: &Stack<T>) -> usize {
        match BinarySpacePartition::to_index(Some(s.clone())) {
            (_, None) => 0,
            (_, Some(x)) => x
        }
    }
}

impl Layout for BinarySpacePartition {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, _: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref st) => {
                debug!("{:?}", st.integrate());
                let ws = st.integrate();

                fn layout(bsp: BinarySpacePartition, l: usize, n: usize) -> Option<BinarySpacePartition> {
                    if l == bsp.size() {
                        Some(bsp.clone())
                    } else if l > bsp.size() {
                        layout(bsp.split_nth(n), l, n)
                    } else {
                        layout(bsp.remove_nth(n), l, n)
                    }
                }

                let bsp = layout(self.clone(), ws.len(), BinarySpacePartition::stack_index(st));;

                let rs = match bsp {
                    None => self.rectangles(screen),
                    Some(ref b) => b.rectangles(screen)
                };
                if let Some(ref t) = bsp.clone() {
                    self.tree =  t.tree.clone();
                }

                ws.into_iter().zip(rs.into_iter()).collect()
            },
            None     => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage, _: &WindowSystem,
        stack: &Option<Stack<Window>>, _: &GeneralConfig) -> bool {
            match message {
                LayoutMessage::TreeRotate => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.rotate_nth(index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }
                },
                LayoutMessage::TreeSwap => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.swap_nth(index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }
                },
                LayoutMessage::TreeExpandTowards(dir) => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.grow_nth_towards(dir, index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }

                },
                LayoutMessage::TreeShrinkFrom(dir) => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.shrink_nth_from(dir, index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }

                },
                _ => false
            }
        }

    fn description(&self) -> String {
        "BSP".to_owned()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}
