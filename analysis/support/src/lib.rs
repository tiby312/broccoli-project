

pub use broccoli;
pub use broccoli::axgeom;
use broccoli::tree::aabb_pin::HasInner;
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
pub trait ColfindHandler:Aabb+ManySwap+HasInner{
    fn handle(a:&mut Self::Inner,b:&mut Self::Inner);
}



pub struct Dummy<T>(pub Rect<f32>,pub T);
impl<T> Aabb for Dummy<T>{
    type Num=f32;
    fn get(&self)->&Rect<f32>{
        &self.0
    }
}
impl<T> Aabb for &mut Dummy<T>{
    type Num=f32;
    fn get(&self)->&Rect<f32>{
        &self.0
    }
}
impl<T> ManySwap for &mut Dummy<T>{}
impl<T> HasInner for &mut Dummy<T>{
    type Inner=T;

    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0,&mut self.1)
    }
}


impl<T> ManySwap for Dummy<T>{}
impl<T> HasInner for Dummy<T>{
    type Inner=T;

    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0,&mut self.1)
    }
}

impl<const K:usize> ColfindHandler for Dummy<&mut [u8;K]>{
    fn handle(a:&mut Self::Inner,b:&mut Self::Inner){

        a[0]^=1;
        b[0]^=1;
    }
}

impl<const K:usize> ColfindHandler for Dummy<[u8;K]>{
    fn handle(a:&mut Self::Inner,b:&mut Self::Inner){

        a[0]^=1;
        b[0]^=1;
    }
}
impl<const K:usize> ColfindHandler for &mut Dummy<[u8;K]>{
    fn handle(a:&mut Self::Inner,b:&mut Self::Inner){

        a[0]^=1;
        b[0]^=1;
    }
}

impl ColfindHandler for Dummy<u32>{
    fn handle(a:&mut Self::Inner,b:&mut Self::Inner){
        
        *a^=1;
        *b^=1;
    }
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

    pub fn dist(grow:f64)->impl Iterator<Item=Rect<f32>>{
        dists::fib_iter([0.0, 0.0], grow).map(|a|{
            axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32))
        }) 
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
