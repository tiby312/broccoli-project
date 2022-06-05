use super::*;
use std::time::Duration;
use std::time::Instant;

pub mod convert {
    //!
    //! The broccoli book mentioned in the root documentation shows that
    //! integer comparisons can be faster than floating point.
    //!
    //! Here are some convenience functions that take a floating point,
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
            convert1d_u32(a.y.start, border.y),
            convert1d_u32(a.y.end, border.y),
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

    #[derive(Debug)]
    pub struct Single {
        level: usize,
        dur: usize,
    }
    #[derive(Debug)]
    pub struct LevelCounter {
        level: usize,
        stuff: Vec<Single>,
        start: usize,
    }
    impl LevelCounter {
        pub fn new(level: usize, buffer: Vec<Single>) -> LevelCounter {
            let now = unsafe { datanum::COUNTER };
            LevelCounter {
                level,
                stuff: buffer,
                start: now,
            }
        }

        fn restart(&mut self, level: usize) {
            let now = unsafe { datanum::COUNTER };
            self.level = level;
            self.start = now;
        }
        pub fn level(&self) -> usize {
            self.level
        }

        pub fn consume(&mut self) {
            let dur = unsafe { datanum::COUNTER - self.start };

            //stop self timer.
            let level = self.level();

            if let Some(a) = self.stuff.iter_mut().find(|x| x.level == level) {
                a.dur += dur;
            } else {
                self.stuff.push(Single {
                    level: self.level,
                    dur,
                });
            }
        }

        pub fn into_levels(mut self) -> Vec<usize> {
            self.consume();
            let mut v: Vec<_> = self.stuff.into_iter().map(|x| x.dur).collect();

            v.reverse();
            let mut n = vec![];

            for i in (0..v.len()).rev() {
                let sum = v[..i + 1].iter().sum();

                n.push(sum);
            }
            n
        }
    }
    impl Splitter for LevelCounter {
        #[inline]
        fn div(&mut self) -> Self {
            let level = self.level();

            self.consume();

            self.restart(level + 1);

            LevelCounter::new(level + 1, vec![])
        }

        #[inline]
        fn add(&mut self, mut b: Self) {
            let l1 = self.level();
            let l2 = b.level();
            assert_eq!(l1, l2);

            self.consume();
            b.consume();

            let v1 = &mut self.stuff;
            let v2 = &mut b.stuff;

            //the left vec is bigger
            for a in v2.into_iter() {
                let b = v1.iter_mut().find(|x| x.level == a.level).unwrap();
                b.dur += a.dur;
            }

            self.restart(l1 - 1);
        }
    }
}

pub use self::leveltimer::LevelTimer;
mod leveltimer {
    use super::*;
    use std::time::Instant;
    #[derive(Debug)]
    pub struct LevelTimer {
        level: usize,
        stuff: Vec<(usize, f64)>,
        start: Instant,
    }

    impl LevelTimer {
        pub fn level(&self) -> usize {
            self.level
        }
        pub fn new(level: usize, data: Vec<(usize, f64)>) -> LevelTimer {
            LevelTimer {
                level,
                stuff: data,
                start: Instant::now(),
            }
        }

        fn restart(&mut self, level: usize) {
            self.level = level;
            self.start = Instant::now();
        }

        pub fn into_levels(mut self) -> Vec<f64> {
            self.consume();
            //self.consume().into_iter().map(|x| x.1).collect()
            let mut v: Vec<_> = self.stuff.into_iter().map(|x| x.1).collect();

            v.reverse();
            let mut n = vec![];

            for i in (0..v.len()).rev() {
                let sum = v[..i + 1].iter().sum();

                n.push(sum);
            }
            n
        }

        pub fn consume(&mut self) {
            let dur = into_secs(self.start.elapsed());
            //stop self timer.
            let level = self.level();
            if let Some(a) = self.stuff.iter_mut().find(|x| x.0 == level) {
                a.1 += dur;
            } else {
                self.stuff.push((self.level, dur));
            }
        }
    }

    impl Splitter for LevelTimer {
        #[inline]
        fn div(&mut self) -> Self {
            let level = self.level();

            self.consume();

            self.restart(level + 1);
            LevelTimer::new(level + 1, vec![])
        }
        #[inline]
        fn add(&mut self, mut b: Self) {
            let l1 = self.level();
            let l2 = b.level();
            assert_eq!(l1, l2);

            self.consume();
            b.consume();

            let v1 = &mut self.stuff;
            let v2 = &mut b.stuff;

            //the left vec is bigger
            for a in v2.into_iter() {
                let b = v1.iter_mut().find(|x| x.0 == a.0).unwrap();
                b.1 += a.1;
            }

            self.restart(l1 - 1);
        }
    }
}

pub fn bench_closure(func: impl FnOnce()) -> f64 {
    black_box_ret(bench_closure_ret(func).1)
}

pub fn bench_closure_ret<T>(func: impl FnOnce() -> T) -> (T, f64) {
    let instant = Instant::now();
    let a = black_box_ret(func());
    let j = instant_to_sec(instant.elapsed());
    (a, j)
}

pub fn instant_to_sec(elapsed: Duration) -> f64 {
    let secs: f64 = elapsed.as_secs() as f64;
    let nano: f64 = elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0
}

pub fn n_iter(start: usize, end: usize) -> core::iter::StepBy<std::ops::Range<usize>> {
    assert!(end > start);
    //hardcode the number of samples
    //because its tied to the graph
    let num_samples = 120;

    let step_size = (end - start) / num_samples;

    (start..end).step_by(step_size)
}

pub fn grow_iter(
    start: f64,
    end: f64,
) -> impl Iterator<Item = f64> + core::iter::DoubleEndedIterator + core::iter::ExactSizeIterator {
    //hardcode the number of samples
    //because it is tied to the graph
    let num_samples = 120;
    let step_size = (end - start) / num_samples as f64;

    (0..num_samples).map(move |x| start + (x as f64 * step_size))
}

pub const RADIUS: f32 = 2.0;

//abspiral(20_000,2.1)~=20_000
//abspiral(20_000,1.5)~=3*20_000
//abspiral(20_000,0.6)~=20*20_000
//abspiral(20_000,0.2)~=180*20_000
pub const DEFAULT_GROW: f64 = 1.5;
pub const DENSE_GROW: f64 = 0.6;
pub const MEGA_DENSE_GROW: f64 = 0.2;
pub const MEGA_MEGA_DENSE_GROW: f64 = 0.02;

pub const SPARSE_GROW: f64 = 2.1;

fn abspiral_f64(grow: f64) -> impl Iterator<Item = Rect<f64>> {
    let s = dists::fib_iter([0.0, 0.0], grow);

    //let s = dists::spiral_iter([0.0, 0.0], 17.0, grow);
    s.map(move |a| axgeom::Rect::from_point(a.into(), vec2same(RADIUS as f64)))
}

pub fn make_tree_ref_ind<N: Num, T>(
    bots: &mut [T],
    grow: f64,
    mut func: impl FnMut(RectConv) -> Rect<N>,
) -> Box<[BBox<N, &mut T>]> {
    let mut k = abspiral_f64(grow);
    bots.iter_mut()
        .map(|a| BBox::new(func(RectConv(k.next().unwrap())), a))
        .collect::<Vec<_>>()
        .into_boxed_slice()
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
        maker.build_from_rect(self.0.inner_as())
    }
    pub fn to_f32dnum(self, maker: &datanum::Maker) -> Rect<datanum::Dnum<f32>> {
        maker.build_from_rect(self.0.inner_as())
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
        r.grow_to_fit(a.get());
    }
    Some(r)
}

pub fn convert_dist<T, T2, X>(
    a: Vec<(Rect<T>, X)>,
    mut func: impl FnMut(Rect<T>) -> Rect<T2>,
) -> Vec<(Rect<T2>, X)> {
    a.into_iter().map(|a| (func(a.0), a.1)).collect()
}

pub fn distribute_iter<T, X>(
    grow: f64,
    i: impl ExactSizeIterator<Item = T> + core::iter::FusedIterator,
    mut func: impl FnMut(RectConv) -> Rect<X>,
) -> Vec<(Rect<X>, T)> {
    abspiral_f64(grow)
        .zip(i)
        .map(|(a, b)| (func(RectConv(a)), b))
        .collect()
}

pub fn distribute<T, X>(
    grow: f64,
    inner: &mut [T],
    mut func: impl FnMut(RectConv) -> Rect<X>,
) -> Vec<(Rect<X>, &mut T)> {
    abspiral_f64(grow)
        .zip(inner.iter_mut())
        .map(|(a, b)| (func(RectConv(a)), b))
        .collect()
}
