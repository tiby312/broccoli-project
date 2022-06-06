

pub use broccoli;
pub use broccoli::axgeom;
pub use poloto;

pub mod prelude{
    pub use super::*;
    pub use poloto::*;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::tree::aabb_pin::AabbPin;
    pub use broccoli::*;
    
    pub use broccoli::tree::node::Aabb;
    pub use broccoli::tree::node::ManySwap;
    pub use broccoli::tree::node::ManySwappable;

}


use prelude::*;

pub mod dist{
    use super::*;
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

    pub fn create_dist_manyswap<K: Clone>(num_bots: usize, grow: f64, k: K) -> Vec<ManySwappable<(Rect<f32>, K)>> {
        let s = dists::fib_iter([0.0, 0.0], grow).take(num_bots);
        s.map(|a| {
            ManySwappable((
                axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32)),
                k.clone(),
            ))
        })
        .collect()
    }


    pub fn create_dist<K>(num_bots: usize, grow: f64, mut func:impl FnMut(&Rect<f32>)->K) -> Vec<(Rect<f32>, K)> {
        let s = dists::fib_iter([0.0, 0.0], grow).take(num_bots);
        s.map(|a| {
            let r=axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32));
            let b=func(&r);
            (
                r,
                b
            )
        })
        .collect()
    }

}


pub fn n_iter(start: usize, end: usize) -> core::iter::StepBy<std::ops::Range<usize>> {
    assert!(end > start);
    //hardcode the number of samples
    //because its tied to the graph
    let num_samples = 120;

    let step_size = (end - start) / num_samples;

    (start..end).step_by(step_size)
}



use std::time::Duration;
use std::time::Instant;

#[inline(never)]
pub fn black_box_ret<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

#[inline(never)]
pub fn black_box<T>(dummy: T) {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        std::mem::forget(ret);
    }
}

pub fn bench_closure(func: impl FnOnce()) -> f64 {
    black_box_ret(bench_closure_ret(func).1)
}

pub fn instant_to_sec(elapsed: Duration) -> f64 {
    let secs: f64 = elapsed.as_secs() as f64;
    let nano: f64 = elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0
}

pub fn bench_closure_ret<T>(func: impl FnOnce() -> T) -> (T, f64) {
    let instant = Instant::now();
    let a = black_box_ret(func());
    let j = instant_to_sec(instant.elapsed());
    (a, j)
}




///
/// Used to wrap a `std::io::Write` to have `std::io::Write`.
/// The underlying error can be extracted through the error field.
///
pub struct Adaptor<T> {
    pub inner: T,
    pub error: Result<(), std::io::Error>,
}


///Update a `std::io::Write` to be a `std::fmt::Write`
pub fn upgrade_write<T: std::io::Write>(inner: T) -> Adaptor<T> {
    Adaptor {
        inner,
        error: Ok(()),
    }
}

impl<T: std::io::Write> std::fmt::Write for Adaptor<T> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self.inner.write_all(s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.error = Err(e);
                Err(std::fmt::Error)
            }
        }
    }
}
