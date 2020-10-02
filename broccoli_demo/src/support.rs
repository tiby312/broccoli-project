pub mod prelude {
    pub use crate::Demo;
    pub use axgeom::*;
    pub use dinotree_alg::analyze;
    pub use dinotree_alg::node::*;
    pub use dinotree_alg::owned::*;
    pub use dinotree_alg::query::*;
    pub use dinotree_alg::*;
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
