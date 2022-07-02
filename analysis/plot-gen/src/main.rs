//TODO remove
#![allow(dead_code)]

mod bench;
mod theory;

use poloto::prelude::*;
use std::path::Path;
use support::datanum::DnumManager;
use support::poloto;
use support::prelude::build::marker::Markerable;
use support::prelude::plotnum::PlotNum;
use support::prelude::ticks::HasDefaultTicks;
use support::prelude::*;

pub trait GraphEmplace {
    fn write_graph<X: PlotNum + HasDefaultTicks, Y: PlotNum + HasDefaultTicks>(
        &mut self,
        group: Option<&str>,
        name: impl std::fmt::Display + std::fmt::Debug,
        x: impl std::fmt::Display,
        y: impl std::fmt::Display,
        plots: impl poloto::build::PlotIterator<X, Y> + Markerable<X, Y>,
        description: &str,
    );
}

mod html {
    use support::{
        poloto,
        prelude::{
            build::marker::Markerable, plotnum::PlotNum, quick_fmt, simple_theme::SimpleTheme,
            ticks::HasDefaultTicks,
        },
    };

    use crate::GraphEmplace;
    use std::{process::CommandEnvs, time::Instant};

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
        fn write_graph<X: PlotNum + HasDefaultTicks, Y: PlotNum + HasDefaultTicks>(
            &mut self,
            group: Option<&str>,
            name: impl std::fmt::Display + std::fmt::Debug,
            x: impl std::fmt::Display,
            y: impl std::fmt::Display,
            plots: impl poloto::build::PlotIterator<X, Y> + Markerable<X, Y>,
            description: &str,
        ) {
            fn try_<K>(a: impl FnOnce() -> K) -> K {
                a()
            }

            try_(|| {
                let name=if let Some(group)=group{
                    format!("{group} : {name}")
                }else{
                    format!("{name}")
                };

                let plotter = poloto::quick_fmt!(&name, x, y, plots,);

                let dd = plotter.get_dim();
                let svg_width = 400.0;
                use poloto::simple_theme;
                let hh = simple_theme::determine_height_from_width(dd, svg_width);

                let mut t = tagger::new(&mut self.w);

                t.elem("div", |w| {
                    w.attr(
                        "style",
                        "width:400px;background:#262626;margin:5px;padding:10px;word-break: normal;white-space: normal;border-radius:6px",
                    )
                })?
                .build(|w| {
                    write!(
                        w.writer_escapable(),
                        "{}<style>{}</style>{}{}",
                        poloto::disp(|a| poloto::simple_theme::write_header(
                            a,
                            [svg_width, hh],
                            dd
                        )),
                        ".poloto_line{stroke-dasharray:2;stroke-width:2;}",
                        poloto::disp(|a| plotter.render(a)),
                        poloto::simple_theme::SVG_END
                    )?;

                    let parser = pulldown_cmark::Parser::new(description);
                    let mut s = String::new();

                    pulldown_cmark::html::push_html(&mut s, parser);

                    w.elem("text", |d| d.attr("class", "markdown-body"))?
                        .build(|w| write!(w.writer_escapable(), "{}", s))
                })
            })
            .unwrap();

            let name = if let Some(group) = group {
                format!("{}:{}", group, name)
            } else {
                name.to_string()
            };

            eprintln!("Elapsed : {:>16?} : {}", self.now.elapsed(), name);
            self.now = Instant::now();
        }
    }
}

/*
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
*/

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
                let mut sys = html::Html::new(w.writer_escapable());

                //let mut a = datanum::new_session();
                //theory::theory(&mut a, path)
                bench::bench(&mut sys);
                Ok(())
            })
        })
}

fn main() {
    foo("../../target/analysis/html").unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}
