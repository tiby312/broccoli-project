use std;
use std::time::Duration;

use crate::inner_prelude::*;
use std::time::Instant;

fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
}

///Measure the time each level of a recursive algorithm takes that supports the Splitter trait.
///Note that the number of elements in the returned Vec could be less than the height of the tree.
///This can happen if the recursive algorithm does not recurse all the way to the leafs because it
///deemed it not necessary.
#[derive(Default)]
pub struct LevelTimer {
    levels: Vec<f64>,
    time: Option<Instant>,
}

impl LevelTimer {
    #[inline]
    pub fn new() -> LevelTimer {
        LevelTimer {
            levels: Vec::new(),
            time: None,
        }
    }

    #[inline]
    pub fn into_inner(self) -> Vec<f64> {
        self.levels
    }
    #[inline]
    fn node_end_common(&mut self) {
        let time = self.time.unwrap();

        let elapsed = time.elapsed();
        self.levels.push(into_secs(elapsed));
        self.time = None;
    }
}
impl Splitter for LevelTimer {
    #[inline]
    fn div(&mut self) -> Self {
        self.node_end_common();

        let length = self.levels.len();

        LevelTimer {
            levels: core::iter::repeat(0.0).take(length).collect(),
            time: None,
        }
    }
    #[inline]
    fn add(&mut self, a: Self) {
        let len = self.levels.len();
        for (a, b) in self.levels.iter_mut().zip(a.levels.iter()) {
            *a += *b;
        }
        if len < a.levels.len() {
            self.levels.extend_from_slice(&a.levels[len..]);
        }
    }
    #[inline]
    fn node_start(&mut self) {
        assert!(self.time.is_none());
        self.time = Some(Instant::now());
    }
    #[inline]
    fn node_end(&mut self) {
        self.node_end_common();
    }
}

pub const COLS: &[&str] = &[
    "blue", "green", "red", "violet", "orange", "brown", "gray", "black", "pink",
];


pub fn bench_closure(func:impl FnOnce())->f64{
    let instant = Instant::now();
    func();
    instant_to_sec(instant.elapsed())
}

pub fn bench_closure_ret<T>(func:impl FnOnce()->T)->(T,f64){
    let instant = Instant::now();
    let a=func();
    (a,instant_to_sec(instant.elapsed()))
}


pub fn instant_to_sec(elapsed: Duration) -> f64 {
    let secs: f64 = elapsed.as_secs() as f64;
    let nano: f64 = elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0
}





pub fn abspiral_grow_iter(range:core::ops::Range<usize>,start:f64,delta:f64)->impl Iterator<Item=f64>{
    range.map(move |a|{
        let a: f64 = a as f64;
        start + a * delta
    })
}

pub const RADIUS:f32=5.0;
pub const ABSPIRAL_PROP:bot::BotProp=bot::BotProp{
    radius: bot::Dist::manual_create(RADIUS,RADIUS*2.0,RADIUS*RADIUS),
    collision_push: 0.1,
    collision_drag: 0.1,
    minimum_dis_sqr: 0.0001,
    viscousity_coeff: 0.1,
};

pub fn abspiral_isize(grow:f64)->impl Iterator<Item=Rect<isize>>{
    abspiral_f64(grow).map(|a|a.inner_as())
}

pub fn abspiral_f32_nan(grow:f64)->impl Iterator<Item=Rect<NotNan<f32>>>{
    abspiral_f32(grow).map(|a|a.inner_try_into().unwrap())
}
pub fn abspiral_f32(grow:f64)->impl Iterator<Item=Rect<f32>>{
    abspiral_f64(grow).map(|a|a.inner_as())
}

pub fn abspiral_f64(grow:f64)->impl Iterator<Item=Rect<f64>>{
    let s = dists::spiral_iter([0.0, 0.0], 17.0, grow as f64); 
    s.map(move |a|{
        let r=axgeom::Rect::from_point(vec2(a[0],a[1]),vec2same(RADIUS as f64));
        r
    })
}