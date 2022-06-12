//TODO remove
#![allow(dead_code)]

mod bench;
mod theory;

use poloto::build::scatter;
use poloto::prelude::*;
use std::path::Path;
use support::datanum::DnumManager;
use support::prelude::*;

pub trait GraphEmplace {
    fn write_graph<J>(
        &mut self,
        group: Option<&str>,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> (J, std::fmt::Result),
    ) -> J;

    fn write_graph_simple<J>(
        &mut self,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> (J, std::fmt::Result),
    ) -> J {
        self.write_graph(None, name, func)
    }

    fn write_graph_group<J>(
        &mut self,
        group: &str,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> (J, std::fmt::Result),
    ) -> J {
        self.write_graph(Some(group), name, func)
    }
}

mod sysfile {
    use super::GraphEmplace;
    use std::{path::Path, time::Instant};

    pub struct SysFile<K> {
        base: K,
        now: Instant,
    }
    impl<K: AsRef<Path>> SysFile<K> {
        pub fn new(path: K) -> Self {
            std::fs::create_dir_all(&path).unwrap();
            SysFile {
                base: path,
                now: Instant::now(),
            }
        }
    }

    impl<K: AsRef<Path>> GraphEmplace for SysFile<K> {
        fn write_graph<J>(
            &mut self,
            group: Option<&str>,
            name: &str,
            func: impl FnOnce(&mut dyn std::fmt::Write) -> (J, std::fmt::Result),
        ) -> J {
            let base = self.base.as_ref();
            let p = if let Some(group) = group {
                std::fs::create_dir_all(base.join(group)).unwrap();
                base.join(group).join(name).with_extension("svg")
            } else {
                base.join(name).with_extension("svg")
            };

            let file = std::fs::File::create(&p).unwrap();

            use crate::poloto::upgrade_write;
            let mut w = upgrade_write(file);
            let (aa, bb) = func(&mut w);

            eprintln!("finish writing:{:?}  elapsed:{:?}", &p, self.now.elapsed());
            self.now = Instant::now();

            bb.unwrap();
            aa
        }
    }
}

fn main() {
    let mut a = datanum::new_session();

    let mut sys = sysfile::SysFile::new("../../target/analysis");

    bench::bench(&mut sys);
}
