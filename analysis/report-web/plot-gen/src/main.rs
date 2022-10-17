use std::path::Path;
use support::datanum::DnumManager;
use support::poloto;
use support::prelude::*;

use hypermelon::build;
use hypermelon::prelude::*;
fn foo<P: AsRef<Path>>(base: P) -> std::fmt::Result {
    let base = base.as_ref();
    std::fs::create_dir_all(base).unwrap();

    let file = std::fs::File::create(base.join("report").with_extension("html")).unwrap();

    let k = hypermelon::build::from_closure_escapable(|w| {
        w.render(build::raw_escapable("<!DOCTYPE html>"))?;

        w.render(
            build::single("meta")
                .with(attrs!(
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1.0")
                ))
                .with_ending(""),
        )?;

        w.session(build::elem("html").with(("style", "background: black;")))
            .build(|w| {
                let style = build::elem("style").append(include_str!("github-markdown.css"));

                let style2 = build::elem("style")
                    .append(poloto::render::Theme::dark().get_str())
                    .append(".poloto_scatter{stroke-width:3}");

                let style = style.chain(style2);

                w.render(style)?;

                w.session(build::elem("div").with((
                    "style",
                    "display:flex;flex-wrap:wrap;justify-content: center;",
                )))
                .build(|w| {
                    let mut c = Custom;
                    let mut j = w.writer_escapable();
                    let mut sys = Html::new(&mut j, &mut c);

                    let mut a = datanum::new_session();
                    handle(&mut sys, &mut a)
                })
            })
    });

    hypermelon::render_escapable(k, hypermelon::tools::upgrade_write(file))
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
        plot: hypermelon::elem::DynamicElement,
        description: &str,
    ) -> std::fmt::Result {
        let div=build::elem("div").with(("style","max-width:400px;width:100%;background:#262626;margin:5px;padding:5px;word-break: normal;white-space: normal;border-radius:10px"));

        let header = build::elem("svg").with(attrs!(
            ("class", "poloto"),
            ("width", "100%"),
            ("viewBox", "0 0 800 500"),
            ("xmlns", "http://www.w3.org/2000/svg")
        ));

        let all = header.append(plot);
        let all = div.append(all);

        let parser = pulldown_cmark::Parser::new(description);
        let mut s = String::new();

        pulldown_cmark::html::push_html(&mut s, parser);
        let text = build::elem("text")
            .with(("class", "markdown-body"))
            .append(build::raw_escapable(s));

        let all = all.append(text);

        hypermelon::render_escapable(all, w)
    }
}
