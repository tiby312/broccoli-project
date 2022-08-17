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
        w.put_raw_escapable(MY_CONFIG)?;
        w.put_raw_escapable(".poloto_scatter{stroke-width:3}")
    })?;

    w.elem("html", |d| d.attr("style", "background: black;"))?
        .build(|w| {
            w.put_raw_escapable(
                r##"<meta name="viewport" content="width=device-width, initial-scale=1.0">"##,
            )?;
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
    best_height::bench(emp)?;
    best_height::theory(emp, man)?;
    best_height::optimal(emp)?;
    levels::bench(emp)?;
    levels::theory(emp, man)?;
    cached_pairs::bench(emp)?;
    float_vs_integer::bench(emp)?;

    rebal_vs_query::bench(emp)?;
    rebal_vs_query::theory(emp, man)?;

    spiral::handle_visualize(emp)?;
    spiral::handle_grow(emp)?;
    spiral::num_intersection(emp)?;

    layout::bench(emp)?;

    // TODO add back
    // par_tuner::bench_par(emp)?;
    // par_tuner::best_seq_fallback_rebal(emp)?;
    // par_tuner::best_seq_fallback_query(emp)?;

    Ok(())
}

fn main() {

    // On my laptop (A chrome acer spin 512 with a octa-core heterogenous cpu),
    // There are 4 cortex A55 and 4 cortex A57 cores.
    // Having these benching threads transfer between the two types of cores
    // causes inconsistent and not smooth performance.
    // lets set the affinity such that the threads only run on the 
    // more powerful a57 cores.
    let worker_cores=[4,5,6,7];

    affinity::set_thread_affinity(worker_cores).unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .start_handler(move |_index| {
            affinity::set_thread_affinity(worker_cores).unwrap();
        })
        .build_global()
        .unwrap();

    foo("../../target/analysis/html").unwrap();
    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}

pub struct Custom;
impl Disper for Custom {
    fn write_graph_disp(
        &mut self,
        w: &mut dyn std::fmt::Write,
        _dim: [f64; 2],
        plot: &mut dyn std::fmt::Display,
        description: &str,
    ) -> std::fmt::Result {
        //let dd = dim;
        //let svg_width = 380.0;
        //TODO remove this kind of thing?
        //let hh = simple_theme::determine_height_from_width(dd, svg_width);

        let mut t = tagger::new(w);

        pub const SVG_HEADER: &str = r##"<svg class="poloto" width="100%" viewBox="0 0 800 500" xmlns="http://www.w3.org/2000/svg">"##;

        t.elem("div", |w| {
            w.attr(
                "style",
                "max-width:400px;width:100%;background:#262626;margin:5px;padding:5px;word-break: normal;white-space: normal;border-radius:10px",
            )
        })?
        .build(|w| {
            write!(
                w.writer_escapable(),
                "{}<style>{}</style>{}{}",
                SVG_HEADER,
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

pub const MY_CONFIG: &str = ".poloto{\
    stroke-linecap:round;\
    stroke-linejoin:round;\
    font-family:Roboto,sans-serif;\
    font-size:16px;\
    }\
    .poloto_background{fill:rgba(0,0,0,0);}\
    .poloto_scatter{stroke-width:7}\
    .poloto_tick_line{stroke:dimgray;stroke-width:0.5}\
    .poloto_line{stroke-width:2}\
    .poloto_text{fill: white;}\
    .poloto_axis_lines{stroke: white;stroke-width:3;fill:none;stroke-dasharray:none}\
    .poloto_title{font-size:24px;dominant-baseline:start;text-anchor:middle;}\
    .poloto_xname{font-size:24px;dominant-baseline:start;text-anchor:middle;}\
    .poloto_yname{font-size:24px;dominant-baseline:start;text-anchor:middle;}\
    .poloto0stroke{stroke:blue;}\
    .poloto1stroke{stroke:red;}\
    .poloto2stroke{stroke:green;}\
    .poloto3stroke{stroke:gold;}\
    .poloto4stroke{stroke:aqua;}\
    .poloto5stroke{stroke:lime;}\
    .poloto6stroke{stroke:orange;}\
    .poloto7stroke{stroke:chocolate;}\
    .poloto0fill{fill:blue;}\
    .poloto1fill{fill:red;}\
    .poloto2fill{fill:green;}\
    .poloto3fill{fill:gold;}\
    .poloto4fill{fill:aqua;}\
    .poloto5fill{fill:lime;}\
    .poloto6fill{fill:orange;}\
    .poloto7fill{fill:chocolate;}";
