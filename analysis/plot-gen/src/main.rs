//TODO remove
#![allow(dead_code)]

mod bench;
mod theory;

use poloto::prelude::*;
use std::path::Path;
use support::datanum::DnumManager;
use support::prelude::*;

pub trait GraphEmplace {
    fn write_graph(
        &mut self,
        group: Option<&str>,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
    );

    fn write_graph_simple(
        &mut self,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
    ) {
        self.write_graph(None, name, func)
    }

    fn write_graph_group(
        &mut self,
        group: &str,
        name: &str,
        func: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
    ) {
        self.write_graph(Some(group), name, func)
    }
}

mod html {
    use crate::GraphEmplace;
    use std::time::Instant;

    pub struct Html<T> {
        w: T,
        now: Instant,
    }

    impl<T: std::fmt::Write> Html<T> {
        pub fn new(w: T) -> Self {
            Html {
                w,
                now: Instant::now(),
            }
        }
    }

    impl<T: std::fmt::Write> GraphEmplace for Html<T> {
        fn write_graph(
            &mut self,
            group: Option<&str>,
            name: &str,
            func: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
        ) {
            func(&mut self.w).unwrap();

            eprintln!(
                "finish writing:{:?}:{:?}  elapsed:{:?}",
                group,
                name,
                self.now.elapsed()
            );
            self.now = Instant::now();
        }
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
        fn write_graph(
            &mut self,
            group: Option<&str>,
            name: &str,
            func: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
        ) {
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
            let aa = func(&mut w);

            eprintln!("finish writing:{:?}  elapsed:{:?}", &p, self.now.elapsed());
            self.now = Instant::now();

            aa.unwrap();
        }
    }
}

fn foo() -> std::fmt::Result {
    //use tagger::no_attr;
    let mut w = tagger::new(tagger::upgrade_write(std::io::stdout()));

    w.put_raw_escapable("<!DOCTYPE html>")?;

    w.elem("html", |d| d.attr("style", "display:flex;flex-wrap:wrap;"))?
        .build(|w| {
            let mut sys = html::Html::new(w.writer_escapable());

            //let mut a = datanum::new_session();
            //theory::theory(&mut a, path)
            bench::bench(&mut sys);
            Ok(())
        })?;

    Ok(())
}

fn main() {
    foo().unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}
