use std;
use std::time::Duration;

use crate::inner_prelude::*;
use std::time::Instant;

pub mod convert {
    //!
    //! The broccoli book mentioned in the root documentation shows that
    //! integer comparisons can be faster than floating point.
    //!
    //! Here are some convinience functions that take a floating point,
    //! and then normalize it over an area as integers
    //!
    //!
    use super::*;

    use axgeom::Rect;

    /*
    ///Convert a `f32` point to a normalized `u32` point normalized over an area.
    #[inline(always)]
    pub fn point_f32_to_u32(a: axgeom::Vec2<f32>, border: &Rect<f32>) -> axgeom::Vec2<u32> {
        axgeom::vec2(convert1d_u32(a.x, border.x), convert1d_u32(a.y, border.y))
    }
    */

    ///Convert a `f32` rect to a normalizde `u32` rect normalized over an area.
    #[inline(always)]
    pub fn rect_f32_to_u32(a: Rect<f32>, border: &Rect<f32>) -> Rect<u32> {
        axgeom::rect(
            convert1d_u32(a.x.start, border.x),
            convert1d_u32(a.x.end, border.x),
            convert1d_u32(a.y.start, border.x),
            convert1d_u32(a.y.end, border.x),
        )
    }

    #[inline(always)]
    fn convert1d_u32(a: f32, range: axgeom::Range<f32>) -> u32 {
        ((a - range.start) * (u32::MAX as f32 / range.distance())) as u32
    }
    /*
    ///Convert a `f32` point to a normalized `u32` point normalized over an area.
    #[inline(always)]
    pub fn point_f32_to_u16(a: axgeom::Vec2<f32>, border: &Rect<f32>) -> axgeom::Vec2<u16> {
        axgeom::vec2(convert1d_u16(a.x, border.x), convert1d_u16(a.y, border.y))
    }
    */


}

fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
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

        /*
        fn leaf_start(&mut self) {
            assert!(self.start.is_none());
            let now = unsafe { datanum::COUNTER };
            self.start = Some(now);
        }
        fn leaf_end(&mut self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(unsafe { datanum::COUNTER - inst });
        }
        */
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
        /*
        fn leaf_start(&mut self) {
            assert!(self.start.is_none());
            let now = Instant::now();
            self.start = Some(now);
        }
        fn leaf_end(&mut self) {
            let inst = self.start.take().unwrap();
            self.stuff.push(into_secs(inst.elapsed()));
        }
        */
    }
}

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


//TODO cache???
pub fn num_intersections_for_grow(grow:f64,num_bot:usize)->usize{
    let mut g:Vec<_>=abspiral_f64(grow).take(num_bot).collect();

    let mut tree=broccoli::new(&mut g);
    let mut num_collision=0;
    tree.find_colliding_pairs_mut(|_,_|{
        num_collision+=1;
    });
    num_collision
}


//TODO use this!!!!
pub fn n_iter(start:usize,end:usize)-> core::iter::StepBy<std::ops::Range<usize>>{
    assert!(end>start);
    //hardcode the number of samples
    //because its tied to the graph
    let num_samples=100;

    let step_size=(end-start)/num_samples;
    
    (start..end).step_by(step_size)
}

//TODO use this!!!!!!!
pub fn grow_iter(start:f64,end:f64)->impl Iterator<Item=f64>+core::iter::DoubleEndedIterator+core::iter::ExactSizeIterator{
    //hardcode the number of samples
    //because it is tied to the graph
    let num_samples=100;
    let step_size=(end-start)/num_samples as f64;

    (0..num_samples).map(move |x|start+(x as f64*step_size))
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
pub const RADIUS: f32 = 5.0;

fn abspiral_f64(grow: f64) -> impl Iterator<Item = Rect<f64>> {
    
    let s = dists::spiral_iter([0.0, 0.0], 17.0, grow);
    s.map(move |a| {
        let r = axgeom::Rect::from_point(a.into(), vec2same(RADIUS as f64));
        r
    })
}

use broccoli::container::*;
pub fn make_tree_ref_ind<'a, 'b, N: Num, T>(
    bots: &'a mut [T],
    grow: f64,
    mut func: impl FnMut(RectConv) -> Rect<N>,
) -> TreeIndBase<'a, N, T> {
    let mut k = abspiral_f64(grow);
    TreeIndBase::new(bots, |_| func(RectConv(k.next().unwrap())))
}

pub struct RectConv(pub Rect<f64>);

impl RectConv {
    pub fn to_f32n(self) -> Rect<f32> {
        self.0.inner_as()
    }
    pub fn to_f64n(self) -> Rect<f64> {
        self.0
    }
    pub fn to_isize_dnum(self, maker: &datanum::Maker) -> Rect<datanum::Dnum<isize>> {
        maker.from_rect(self.0.inner_as())
    }
    pub fn to_f32dnum(self, maker: &datanum::Maker) -> Rect<datanum::Dnum<f32>> {
        maker.from_rect(self.0.inner_as())
    }

    pub fn to_i32(self) -> Rect<i32> {
        self.0.inner_as()
    }

    pub fn to_i64(self) -> Rect<i64> {
        self.0.inner_as()
    }
}

pub fn compute_border<T: Aabb>(bb: &[T]) -> Option<Rect<T::Num>> {
    let (first, rest) = bb.split_first()?;
    let mut r = *first.get();
    for a in rest.iter() {
        r.grow_to_fit(&a.get());
    }
    Some(r)
}
pub fn convert_dist<T, T2, X>(
    a: Vec<BBox<T, X>>,
    mut func: impl FnMut(Rect<T>) -> Rect<T2>,
) -> Vec<BBox<T2, X>> {
    a.into_iter().map(|a| bbox(func(a.rect), a.inner)).collect()
}

pub fn distribute_iter<'a, T, X>(
    grow: f64,
    i: impl ExactSizeIterator<Item = T> + core::iter::FusedIterator,
    mut func: impl FnMut(RectConv) -> Rect<X>,
) -> Vec<BBox<X, T>> {
    abspiral_f64(grow)
        .zip(i)
        .map(|(a, b)| bbox(func(RectConv(a)), b))
        .collect()
}

pub fn distribute<'a, T, X>(
    grow: f64,
    inner: &'a mut [T],
    mut func: impl FnMut(RectConv) -> Rect<X>,
) -> Vec<BBox<X, &'a mut T>> {
    abspiral_f64(grow)
        .zip(inner.iter_mut())
        .map(|(a, b)| bbox(func(RectConv(a)), b))
        .collect()
}



