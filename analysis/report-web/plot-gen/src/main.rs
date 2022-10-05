use std::path::Path;
use hypermelon::elem::RenderElem;
use support::datanum::DnumManager;
use support::poloto;
use support::prelude::*;

use hypermelon::build;
use hypermelon::prelude::*;
fn foo<P: AsRef<Path>>(base: P) -> std::fmt::Result {
    let base = base.as_ref();
    std::fs::create_dir_all(base).unwrap();

    let file = std::fs::File::create(base.join("report").with_extension("html")).unwrap();

    //use tagger::no_attr;
    //let mut w = tagger::new(tagger::upgrade_write(file));

    let header = build::raw_escapable("<!DOCTYPE html>");

    let style = build::elem("style")
        .append(include_str!("github-markdown.css"))
        .append(MY_CONFIG)
        .append(".poloto_scatter{stroke-width:3}");

    let html = build::elem("html").with(("style", "background: black;"));

    let s = build::raw_escapable(
        r##"<meta name="viewport" content="width=device-width, initial-scale=1.0">"##,
    );

    let div = build::elem("div").with((
        "style",
        "display:flex;flex-wrap:wrap;justify-content: center;",
    ));

    let special = build::from_closure(|w| {
        let mut c = Custom;
        let mut j=w.writer_escapable();
        let mut sys = Html::new(&mut j, &mut c);

        let mut a = datanum::new_session();
        handle(&mut sys, &mut a)
    });

    let div = div.append(special);
    let html = html.append(s).append(div);
    let all = header.append(style).append(html);

    hypermelon::render(all, hypermelon::tools::upgrade_write(file))
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
    par_tuner::bench_par(emp)?;
    par_tuner::best_seq_fallback_rebal(emp)?;
    par_tuner::best_seq_fallback_query(emp)?;

    Ok(())
}

fn main() {
    // On my laptop (A chrome acer spin 512 with a octa-core heterogenous cpu),
    // There are 4 cortex A55 and 4 cortex A57 cores.
    // Having these benching threads transfer between the two types of cores
    // causes inconsistent and not smooth performance.
    // lets set the affinity such that the threads only run on the
    // more powerful a57 cores.
    let worker_cores = [4, 5, 6, 7];

    affinity::set_thread_affinity(worker_cores).unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .start_handler(move |index| {
            affinity::set_thread_affinity([worker_cores[index]]).unwrap();
        })
        .build_global()
        .unwrap();

    rayon::scope(|s| {
        s.spawn(|_| {
            foo("../../target/analysis/html").unwrap();
        });
    });

    //let mut sys = sysfile::SysFile::new("../../target/analysis");
    //bench::bench(&mut sys);
}

pub struct Custom;
impl Disper for Custom {
    fn write_graph_disp(
        &mut self,
        w: &mut dyn std::fmt::Write,
        _dim: [f64; 2],
        plot: hypermelon::elem::DynElem,
        description: &str,
    ) -> std::fmt::Result {
        //let dd = dim;
        //let svg_width = 380.0;
        //TODO remove this kind of thing?
        //let hh = simple_theme::determine_height_from_width(dd, svg_width);

        let header = poloto::header().with(("width", "100%"));

        let div=build::elem("div").with(("style","max-width:400px;width:100%;background:#262626;margin:5px;padding:5px;word-break: normal;white-space: normal;border-radius:10px"));

        let style = build::elem("style").append(".poloto_line{stroke-dasharray:2;stroke-width:2;}");

        let all = header
            .append(div)
            .append(style)
            .append(plot);

        let parser = pulldown_cmark::Parser::new(description);
        let mut s = String::new();

        pulldown_cmark::html::push_html(&mut s, parser);
        let text = build::elem("text")
            .with(("class", "markdown-body"))
            .append(build::raw_escapable(s));

        let all = all.append(text);
        //TODO return elem instead of writing
        hypermelon::render(all, w)
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
