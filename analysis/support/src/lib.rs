pub mod datanum;

pub use broccoli;
pub use broccoli::axgeom;
use broccoli::tree::aabb_pin::HasInner;
use broccoli::tree::node::Num;
pub use indoc;
pub use poloto;
pub use tagger;
pub mod prelude {
    pub use super::*;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::tree::aabb_pin::AabbPin;
    pub use broccoli::*;
    pub use indoc::formatdoc;
    pub use poloto::prelude::*;
    pub use poloto::*;

    pub use broccoli::tree::node::Aabb;
    pub use broccoli::tree::node::ManySwap;
    pub use broccoli::tree::node::ManySwappable;
    pub use datanum::DnumManager;
}
pub trait ColfindHandler: Aabb + ManySwap + HasInner {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>);
}

pub trait Recorder<L> {
    fn time_ext<K>(&mut self, func: impl FnOnce() -> K) -> (K, L);

    fn time(&mut self, func: impl FnOnce()) -> L {
        self.time_ext(func).1
    }
}

pub struct Bencher;
impl Recorder<f64> for Bencher {
    fn time_ext<K>(&mut self, func: impl FnOnce() -> K) -> (K, f64) {
        let instant = Instant::now();
        let a = black_box_ret(func());
        let j = instant_to_sec(instant.elapsed());
        (a, j)
    }
}

pub struct Dummy<I, T>(pub Rect<I>, pub T);
impl<I: Num, T> Aabb for Dummy<I, T> {
    type Num = I;
    fn get(&self) -> &Rect<I> {
        &self.0
    }
}
impl<I: Num, T> Aabb for &mut Dummy<I, T> {
    type Num = I;
    fn get(&self) -> &Rect<I> {
        &self.0
    }
}
impl<I, T> ManySwap for &mut Dummy<I, T> {}
impl<I: Num, T> HasInner for &mut Dummy<I, T> {
    type Inner = T;

    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0, &mut self.1)
    }
}

impl<I, T> ManySwap for Dummy<I, T> {}
impl<I: Num, T> HasInner for Dummy<I, T> {
    type Inner = T;

    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0, &mut self.1)
    }
}

impl<const K: usize> ColfindHandler for Dummy<f32, &mut [u8; K]> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        a.unpack_inner()[0] ^= 1;
        b.unpack_inner()[0] ^= 1;
    }
}

impl<const K: usize> ColfindHandler for Dummy<f32, [u8; K]> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        a.unpack_inner()[0] ^= 1;
        b.unpack_inner()[0] ^= 1;
    }
}
impl<const K: usize> ColfindHandler for &mut Dummy<f32, [u8; K]> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        a.unpack_inner()[0] ^= 1;
        b.unpack_inner()[0] ^= 1;
    }
}

impl ColfindHandler for Dummy<datanum::Dnum<f32>, u32> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        *a.unpack_inner() ^= 1;
        *b.unpack_inner() ^= 1;
    }
}

impl ColfindHandler for Dummy<f32, u32> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        *a.unpack_inner() ^= 1;
        *b.unpack_inner() ^= 1;
    }
}

impl ColfindHandler for Dummy<u32, u32> {
    fn handle(a: AabbPin<&mut Self>, b: AabbPin<&mut Self>) {
        *a.unpack_inner() ^= 1;
        *b.unpack_inner() ^= 1;
    }
}

use poloto::build::marker::Markerable;
use poloto::plotnum::PlotNum;
use poloto::ticks::HasDefaultTicks;
use prelude::*;

pub mod dist {
    use crate::datanum::Dnum;

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

    pub fn dist(grow: f64) -> impl Iterator<Item = Rect<f32>> {
        dists::fib_iter([0.0, 0.0], grow).map(|a| {
            axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32))
        })
    }

    pub fn dist_datanum(
        man: &mut datanum::DnumManager,
        grow: f64,
    ) -> impl Iterator<Item = Rect<Dnum<f32>>> + '_ {
        dists::fib_iter([0.0, 0.0], grow).map(|a| {
            let r =
                axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32));
            man.convert(r)
        })
    }
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

//TODO group time stuff?
pub fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
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

pub struct Html<'a> {
    w: &'a mut dyn std::fmt::Write,
    disper: &'a mut dyn Disper,
    now: Instant,
}

pub trait Disper {
    fn write_graph_disp(
        &mut self,
        write: &mut dyn std::fmt::Write,
        dim: [f64; 2],
        plot: &mut dyn std::fmt::Display,
        description: &str,
    ) -> std::fmt::Result;
}

impl<'a> Html<'a> {
    pub fn new<T: std::fmt::Write>(w: &'a mut T, disper: &'a mut dyn Disper) -> Self {
        Html {
            w,
            now: Instant::now(),
            disper,
        }
    }

    pub fn write_graph<X: PlotNum + HasDefaultTicks, Y: PlotNum + HasDefaultTicks>(
        &mut self,
        group: Option<&str>,
        name: impl std::fmt::Display,
        x: impl std::fmt::Display,
        y: impl std::fmt::Display,
        plots: impl poloto::build::PlotIterator<X, Y> + Markerable<X, Y>,
        description: &str,
    ) -> std::fmt::Result {
        let render_opt = poloto::render::render_opt();
        self.write_graph_ext(render_opt, group, name, x, y, plots, description)
    }

    pub fn write_graph_ext<X: PlotNum + HasDefaultTicks, Y: PlotNum + HasDefaultTicks>(
        &mut self,
        render_opt: poloto::render::RenderOptions,
        group: Option<&str>,
        name: impl std::fmt::Display,
        x: impl std::fmt::Display,
        y: impl std::fmt::Display,
        plots: impl poloto::build::PlotIterator<X, Y> + Markerable<X, Y>,
        description: &str,
    ) -> std::fmt::Result {
        let name = if let Some(group) = group {
            format!("{}:{}", group, name)
        } else {
            name.to_string()
        };
        let plotter = poloto::quick_fmt_opt!(render_opt, &name, x, y, plots,);
        let dd = plotter.get_dim();
        let mut disp = poloto::disp(|x| plotter.render(x));
        self.disper
            .write_graph_disp(self.w, dd, &mut disp, description)?;

        eprintln!("{:<10.2?} : {}", self.now.elapsed(), name);
        self.now = Instant::now();
        Ok(())
    }
}
