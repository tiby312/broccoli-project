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
}
