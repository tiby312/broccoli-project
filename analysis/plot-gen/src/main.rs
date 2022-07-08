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
                let mut c = Custom;
                let mut sys = Html::new(w.writer_escapable(), &mut c);

                let mut a = datanum::new_session();
                handle(&mut sys, &mut a)?;
                Ok(())
            })
        })
}

pub fn handle(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    colfind::theory(emp, man)?;
    colfind::bench(emp)?;
    colfind::bench_grow(emp)?;
    colfind::theory_grow(emp, man)?;
    best_height::optimal(emp)?;
    best_height::bench(emp)?;
    cached_pairs::bench(emp)?;
    float_vs_integer::bench(emp)?;
    layout::bench(emp)?;
    par_tuner::bench_par(emp)?;
    par_tuner::best_seq_fallback_rebal(emp)?;
    par_tuner::best_seq_fallback_query(emp)?;
    Ok(())
}

fn main() {
    foo("../../target/analysis/html").unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}

pub struct Custom;
impl Disper for Custom {
    fn write_graph_disp(
        &mut self,
        w: &mut dyn std::fmt::Write,
        dim: [f64; 2],
        plot: &mut dyn std::fmt::Display,
        description: &str,
    ) -> std::fmt::Result {
        let dd = dim;
        let svg_width = 400.0;
        let hh = simple_theme::determine_height_from_width(dd, svg_width);

        let mut t = tagger::new(w);

        t.elem("div", |w| {
            w.attr(
                "style",
                "width:400px;background:#262626;margin:5px;padding:10px;word-break: normal;white-space: normal;border-radius:8px",
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
                plot,
                poloto::simple_theme::SVG_END
            )?;

            let parser = pulldown_cmark::Parser::new(description);
            let mut s = String::new();

            pulldown_cmark::html::push_html(&mut s, parser);

            w.elem("text", |d| d.attr("class", "markdown-body"))?
                .build(|w| write!(w.writer_escapable(), "{}", s))
        })
    }
}
