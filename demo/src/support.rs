pub mod prelude {
    pub use crate::Demo;
    pub use broccoli::axgeom;
    pub use broccoli::axgeom::*;
    pub use broccoli::compt;
    pub use broccoli::prelude::*;
    pub use broccoli::rayon;
    pub use broccoli::*;

    //pub use dists;
    pub use crate::dists::*;
    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
    pub use duckduckgeo::bot::*;
    pub use duckduckgeo::F32n;
    pub use duckduckgeo::*;
    pub use egaku2d::*;
    pub use ordered_float::NotNan;
    pub use crate::*;
}


use broccoli::*;
use axgeom::ordered_float::NotNan;
pub fn point_to_rect_f32(a:axgeom::Vec2<f32>,radius:f32)->Rect<NotNan<f32>>{
    Rect::from_point(a,axgeom::vec2same(radius)).inner_try_into().unwrap()
}

pub fn distribute<X,T:Num>(inner:&mut [X],mut func:impl FnMut(&X)->Rect<T>)->Vec<BBox<T,&mut X>>{
    inner.iter_mut().map(|a|bbox(func(a),a)).collect()
}