//TODO remove
#![allow(dead_code)]

mod bench;

use poloto::prelude::*;
use std::path::Path;
use support::datanum::DnumManager;
use support::poloto;
use support::prelude::*;

fn foo<P: AsRef<Path>>(base: P) -> std::fmt::Result {
    let base = base.as_ref();
    std::fs::create_dir_all(base).unwrap();

    let file = std::fs::File::create(base.join("report").with_extension("html")).unwrap();

    //use tagger::no_attr;
    let mut w = tagger::new(tagger::upgrade_write(file));

    w.put_raw_escapable("<!DOCTYPE html>")?;

    w.elem("style", tagger::no_attr())?
        .build(|w| w.put_raw(include_str!("github-markdown.css")))?;

    w.elem("style", tagger::no_attr())?.build(|w| {
        w.put_raw_escapable(poloto::simple_theme::STYLE_CONFIG_DARK_DEFAULT)?;
        w.put_raw_escapable(".poloto_scatter{stroke-width:3}")
    })?;

    w.elem("html", |d| d.attr("style", "background: black;"))?
        .build(|w| {
            w.elem("div", |d| {
                d.attr(
                    "style",
                    "display:flex;flex-wrap:wrap;justify-content: center;",
                )
            })?
            .build(|w| {
                let mut sys = Html::new(w.writer_escapable());

                let mut a = datanum::new_session();
                //theory::theory(&mut a, path)
                bench::bench(&mut sys, &mut a);
                Ok(())
            })
        })
}

fn main() {
    foo("../../target/analysis/html").unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}
