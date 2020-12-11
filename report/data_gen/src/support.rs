use std;
use std::time::Duration;

use crate::inner_prelude::*;
use std::time::Instant;

fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
}

pub fn bool_then<T>(a: bool, mut func: impl FnMut() -> T) -> Option<T> {
    if a {
        Some(func())
    } else {
        None
    }
}

pub use self::levelcounter::LevelCounter;
mod levelcounter {
    use super::*;

    pub struct LevelCounter {
        stuff: Vec<usize>,
        start: Option<usize>,
    }
    impl LevelCounter {
        pub fn new() -> LevelCounter {
            LevelCounter {
                stuff: Vec::new(),
                start: None,
            }
        }

        pub fn into_tree(
            self,
        ) -> compt::dfs_order::CompleteTreeContainer<usize, compt::dfs_order::PreOrder> {
            let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(self.stuff).unwrap();
            tree
        }

        pub fn into_inorder_dfs(self) -> Vec<usize> {
            let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(self.stuff).unwrap();
            use compt::Visitor;

            let vals: Vec<_> = tree.vistr().dfs_inorder_iter().map(|a| *a).collect();
            vals
        }
        pub fn into_levels(self) -> Vec<usize> {
            let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(self.stuff).unwrap();

            use compt::Visitor;
            let mut times: Vec<_> = core::iter::repeat(0).take(tree.get_height()).collect();
            for (depth, a) in tree.vistr().with_depth(compt::Depth(0)).dfs_preorder_iter() {
                times[depth.0] += a;
            }

            times
        }
    }
    impl Splitter for LevelCounter {
        #[inline]
        fn div(&mut self) -> (Self, Self) {
            assert!(self.start.is_none());
            let now = unsafe { datanum::COUNTER };
            self.start = Some(now);
            (
                LevelCounter {
                    stuff: Vec::new(),
                    start: None,
                },
                LevelCounter {
                    stuff: Vec::new(),
                    start: None,
                },
            )
        }
        #[inline]
        fn add(&mut self, mut a: Self, mut b: Self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(unsafe { datanum::COUNTER - inst });
            self.stuff.append(&mut a.stuff);
            self.stuff.append(&mut b.stuff);
        }

        fn leaf_start(&mut self) {
            assert!(self.start.is_none());
            let now = unsafe { datanum::COUNTER };
            self.start = Some(now);
        }
        fn leaf_end(&mut self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(unsafe { datanum::COUNTER - inst });
        }
    }
}

pub use self::leveltimer::LevelTimer;
mod leveltimer {
    use super::*;
    use std::time::Instant;
    pub struct LevelTimer {
        stuff: Vec<f64>,
        start: Option<Instant>,
    }

    impl LevelTimer {
        pub fn new() -> LevelTimer {
            LevelTimer {
                stuff: Vec::new(),
                start: None,
            }
        }
        pub fn into_levels(self) -> Vec<f64> {
            let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(self.stuff).unwrap();

            use compt::Visitor;
            let mut times: Vec<_> = core::iter::repeat(0.0).take(tree.get_height()).collect();
            for (depth, a) in tree.vistr().with_depth(compt::Depth(0)).dfs_preorder_iter() {
                if depth.0 < tree.get_height() {
                    times[depth.0] += a;
                }
            }

            times
        }
    }
    impl Splitter for LevelTimer {
        #[inline]
        fn div(&mut self) -> (Self, Self) {
            assert!(self.start.is_none());
            let now = Instant::now();

            //self.stuff.push(0.0);
            self.start = Some(now);
            (
                LevelTimer {
                    stuff: Vec::new(),
                    start: None,
                },
                LevelTimer {
                    stuff: Vec::new(),
                    start: None,
                },
            )
        }
        #[inline]
        fn add(&mut self, mut a: Self, mut b: Self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(into_secs(inst.elapsed()));
            self.stuff.append(&mut a.stuff);
            self.stuff.append(&mut b.stuff);
        }

        fn leaf_start(&mut self) {
            assert!(self.start.is_none());
            let now = Instant::now();
            self.start = Some(now);
        }
        fn leaf_end(&mut self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(into_secs(inst.elapsed()));
        }
    }
}

pub const COLS: &[&str] = &[
    "blue", "green", "red", "violet", "orange", "brown", "gray", "black", "pink",
];

pub fn bench_closure(func: impl FnOnce()) -> f64 {
    black_box(bench_closure_ret(func).1)
}

pub fn bench_closure_ret<T>(func: impl FnOnce() -> T) -> (T, f64) {
    let instant = Instant::now();
    let a = black_box(func());
    let j = instant_to_sec(instant.elapsed());
    (a, j)
}

pub fn instant_to_sec(elapsed: Duration) -> f64 {
    let secs: f64 = elapsed.as_secs() as f64;
    let nano: f64 = elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0
}

pub fn abspiral_grow_iter2(start: f64, end: f64, delta: f64) -> impl Iterator<Item = f64> {
    let mut c = start;
    core::iter::from_fn(move || {
        if c >= end {
            None
        } else {
            let k = c;
            c += delta;
            Some(k)
        }
    })
}

/*
#[deprecated(
    note = "abspiral_grow_iter2"
)]
pub fn abspiral_grow_iter(range:core::ops::Range<usize>,start:f64,delta:f64)->impl Iterator<Item=f64>{
    range.map(move |a|{
        let a: f64 = a as f64;
        start + a * delta
    })
}
*/

pub const RADIUS: f32 = 5.0;
pub const ABSPIRAL_PROP: bot::BotProp = bot::BotProp {
    radius: bot::Dist::manual_create(RADIUS, RADIUS * 2.0, RADIUS * RADIUS),
    collision_push: 0.1,
    collision_drag: 0.1,
    minimum_dis_sqr: 0.0001,
    viscousity_coeff: 0.1,
};

pub fn abspiral_datanum<'a>(
    maker: &'a datanum::Maker,
    grow: f64,
) -> impl Iterator<Item = Rect<datanum::Dnum<'a, isize>>> {
    abspiral_f64(grow)
        .map(|a| a.inner_as::<isize>())
        .map(move |a| maker.from_rect(a))
}

pub fn abspiral_datanum_f32_nan<'a>(
    maker: &'a datanum::Maker,
    grow: f64,
) -> impl Iterator<Item = Rect<datanum::Dnum<'a, NotNan<f32>>>> {
    abspiral_f32_nan(grow).map(move |a| maker.from_rect(a))
}

pub fn abspiral_f32_nan(grow: f64) -> impl Iterator<Item = Rect<NotNan<f32>>> {
    abspiral_f32(grow).map(|a| a.inner_try_into().unwrap())
}
pub fn abspiral_f32(grow: f64) -> impl Iterator<Item = Rect<f32>> {
    abspiral_f64(grow).map(|a| a.inner_as())
}

pub fn abspiral_f64(grow: f64) -> impl Iterator<Item = Rect<f64>> {
    let s = dists::spiral_iter([0.0, 0.0], 17.0, grow as f64);
    s.map(move |a| {
        let r = axgeom::Rect::from_point(vec2(a[0], a[1]), vec2same(RADIUS as f64));
        r
    })
}
