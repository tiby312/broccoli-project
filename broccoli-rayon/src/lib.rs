#![forbid(unsafe_code)]

pub mod build;
pub mod queries;

pub mod prelude {
    pub use super::build::RayonBuildPar;
    pub use super::queries::colfind::RayonQueryPar;
}
