pub mod prelude {
    pub use crate::dists::*;
    pub use crate::*;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::compt;
    pub use broccoli::tree::bbox;
    pub use broccoli::tree::node::*;
    pub use broccoli::tree::*;
    //pub use broccoli::rayon;

    pub use broccoli::prelude::*;

    pub use crate::demos::Demo;
    pub use crate::demos::DemoData;
    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
    pub use duckduckgeo::*;
    pub use shogo::simple2d::CtxWrap;
    pub use shogo::simple2d::Shapes;
}

use axgeom::*;
use broccoli::tree::bbox;
use broccoli::tree::node::*;

fn rand() -> f32 {
    unsafe { js_sys::Math::random() as f32 }
}

pub fn make_rand(border: Rect<f32>) -> impl Iterator<Item = [f32; 2]> + Clone + Send + Sync {
    std::iter::repeat_with(move || {
        let randx = rand();
        let randy = rand();

        let xx = border.x.start + randx * border.x.end;
        let yy = border.y.start + randy * border.y.end;
        [xx, yy]
    })
}

pub fn make_rand_rect(
    border: Rect<f32>,
    radius: [f32; 2],
) -> impl Iterator<Item = Rect<f32>> + Clone + Send + Sync {
    std::iter::repeat_with(move || {
        let randx = rand();
        let randy = rand();
        let radiusr = rand();

        let xx = border.x.start + randx * border.x.end;
        let yy = border.y.start + randy * border.y.end;
        let radius = radius[0] + (radius[1] - radius[0]) * radiusr;
        Rect::from_point(vec2(xx, yy), vec2same(radius))
    })
}

use broccoli::*;
pub fn point_to_rect_f32(a: axgeom::Vec2<f32>, radius: f32) -> Rect<f32> {
    Rect::from_point(a, axgeom::vec2same(radius))
}

pub fn distribute<X, T: Num>(
    inner: &mut [X],
    mut func: impl FnMut(&X) -> Rect<T>,
) -> Vec<BBox<T, &mut X>> {
    inner.iter_mut().map(|a| bbox(func(a), a)).collect()
}
