pub mod prelude {
    pub use crate::dists::*;
    pub use crate::Demo;
    pub use crate::*;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::bbox;
    pub use broccoli::build::*;
    pub use broccoli::compt;
    pub use broccoli::node::*;
    pub use broccoli::prelude::*;
    pub use broccoli::RayonJoin;
    //pub use broccoli::rayon;
    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
    pub use duckduckgeo::*;
    pub use egaku2d::*;
}

use axgeom::*;
use broccoli::node::*;

pub fn make_rand<T>(num: usize, border: Rect<f32>, mut func: impl FnMut(Vec2<f32>) -> T) -> Vec<T> {
    crate::dists::rand2_iter(border)
        .map(|[a, b]| axgeom::vec2(a as f32, b as f32))
        .map(|a| func(a))
        .take(num)
        .collect()
}

pub fn make_rand_rect<T>(
    num: usize,
    border: Rect<f32>,
    radius: [f32; 2],
    mut func: impl FnMut(Rect<f32>) -> T,
) -> Vec<T> {
    crate::dists::rand2_iter(border)
        .zip(crate::dists::rand_iter(radius[0], radius[1]))
        .map(|([x, y], radius)| Rect::from_point(vec2(x as f32, y as f32), vec2same(radius as f32)))
        .map(|a| func(a))
        .take(num)
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
