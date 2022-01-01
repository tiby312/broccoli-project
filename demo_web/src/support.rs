pub mod prelude {
    pub use crate::dists::*;
    pub use crate::*;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::bbox;
    pub use broccoli::build::*;
    pub use broccoli::compt;
    pub use broccoli::node::*;
    pub use broccoli::par::RayonJoin;
    pub use broccoli::query::*;
    //pub use broccoli::rayon;
    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
    pub use duckduckgeo::*;
    pub use shogo::dots::CtxExt;
    pub use shogo::dots::Shapes;
    pub use crate::demos::Demo;
}

use axgeom::*;
use broccoli::node::*;

pub fn make_bots<T>(num: usize, border: Rect<f32>, func: impl FnMut(Vec2<f32>) -> T) -> Vec<T> {

    crate::dists::grid::Grid::new(border, num)
        .map(func)
        .collect()
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
