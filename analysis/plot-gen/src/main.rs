//TODO remove
#![allow(dead_code)]

mod bench;
mod theory;

use std::path::Path;

use poloto::build::scatter;
use poloto::prelude::*;
use support::datanum::DnumManager;
use support::prelude::*;

fn main() {
    let mut a = datanum::new_session();

    let p = Path::new("../../target/analysis");
    std::fs::create_dir_all(p).unwrap();
    //theory::theory(&mut a, p);
    bench::bench(p);
}
