use crate::inner_prelude::*;
use broccoli::pmut::PMut;

mod bench;
mod theory;
pub use bench::handle_bench;
pub use theory::handle_theory;
