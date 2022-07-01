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
        description:&str
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
    use std::{time::Instant, process::CommandEnvs};

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
            description:&str
        ) {


            //let k = quick_fmt!(&name, x, y, plots);

            //k.simple_theme_dark(&mut self.w).unwrap();

            //Make the plotting area slightly larger.

            //pub const CUSTOM_SVG: &str = r####"<svg class="poloto_background poloto" width="300px" height="100%" viewBox="0 0 800 500" xmlns="http://www.w3.org/2000/svg">"####;
            let plotter = poloto::quick_fmt!(&name, x, y, plots,);

            let dd = plotter.get_dim();
            let svg_width = 400.0;
            use poloto::simple_theme;
            let hh = simple_theme::determine_height_from_width(dd, svg_width);

            write!(
                &mut self.w,
                "{}<style>{}{}</style>{}{}",
                poloto::disp(|a| poloto::simple_theme::write_header(a, [svg_width, hh], dd)),
                poloto::simple_theme::STYLE_CONFIG_DARK_DEFAULT,
                ".poloto_line{stroke-dasharray:2;stroke-width:2;}",
                poloto::disp(|a| plotter.render(a)),
                poloto::simple_theme::SVG_END
            )
            .unwrap();


            let parser = pulldown_cmark::Parser::new(description);
            let mut s=String::new();

            pulldown_cmark::html::push_html(&mut s,parser);
            write!(&mut self.w,"{}",s).unwrap();

            // write!(
            //     &mut self.w,
            //     "{}<style>{}{}</style>{}{}",
            //     CUSTOM_SVG,
            //     poloto::simple_theme::STYLE_CONFIG_DARK_DEFAULT,
            //     ".poloto_scatter{stroke-width:20}",
            //     poloto::disp(|a| {
            //         let s = poloto::quick_fmt!(
            //             &name,
            //             x,
            //             y,
            //             plots,
            //         );
            //         s.render(a)
            //     }),
            //     poloto::simple_theme::SVG_END
            // ).unwrap();

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

    w.elem("html", |d| {
        d.attr(
            "style",
            "display:flex;flex-wrap:wrap;background-color: #262626;",
        )
    })?
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
    foo("../../target/analysis/html").unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}
