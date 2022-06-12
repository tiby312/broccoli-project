use support::prelude::build::histogram;

use super::*;

pub fn bench(path: &Path) {
    best_height(path);
    /*
    colfind(path);
    layout(path);
    parallel(path);
    float_vs_integer(path);
    rebal_vs_query(path);
    */
}

fn best_height(path: &Path) {
    let num = 30_000;
    let l = broccoli::tree::BuildArgs::new(num);
    let res = best_height::bench(num, 3, l.num_level + 4, 2.0);

    let l1 = scatter("", res.iter().map(|&(i, r)| (i as i128, r)));

    let m = poloto::build::markers([], [0.0]);
    let data = plots!(l1, m);

    let p = simple_fmt!(data, "best-height", "height", "time");

    let mut file = std::fs::File::create(path.join("best-height.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}

fn colfind(path: &Path) {
    std::fs::create_dir_all(path.join("colfind")).unwrap();

    for grow in [2.0] {
        let res = colfind::bench(60_000, grow, 10000, 20000);

        let l1 = scatter("brocc", res.iter().map(|(i, r)| (*i as i128, r.brocc)));
        let l2 = scatter(
            "brocc_par",
            res.iter().map(|(i, r)| (*i as i128, r.brocc_par)),
        );
        let l3 = scatter("nosort", res.iter().map(|(i, r)| (*i as i128, r.nosort)));
        let l4 = scatter(
            "nosort_par",
            res.iter().map(|(i, r)| (*i as i128, r.nosort_par)),
        );
        let l5 = scatter("sweep", res.iter().map(|(i, r)| (*i as i128, r.sweep)));
        let l6 = scatter(
            "sweep_par",
            res.iter().map(|(i, r)| (*i as i128, r.sweep_par)),
        );
        let l7 = scatter("naive", res.iter().map(|(i, r)| (*i as i128, r.naive)));

        let m = poloto::build::origin();
        let data = plots!(l1, l2, l3, l4, l5, l6, l7, m);

        let p = simple_fmt!(data, "hay", "x", "y");

        let mut file =
            std::fs::File::create(path.join("colfind").join(format!("n_{}.svg", grow))).unwrap();

        p.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }

    for n in [60_000] {
        let res = colfind::bench_grow(n, 0.2, 1.5);

        let l1 = scatter("brocc", res.iter().map(|(i, r)| (*i, r.brocc)));
        let l2 = scatter("brocc_par", res.iter().map(|(i, r)| (*i, r.brocc_par)));
        let l3 = scatter("nosort", res.iter().map(|(i, r)| (*i, r.nosort)));
        let l4 = scatter("nosort_par", res.iter().map(|(i, r)| (*i, r.nosort_par)));
        let l5 = scatter("sweep", res.iter().map(|(i, r)| (*i, r.sweep)));
        let l6 = scatter("sweep_par", res.iter().map(|(i, r)| (*i, r.sweep_par)));
        let l7 = scatter("naive", res.iter().map(|(i, r)| (*i, r.naive)));

        let m = poloto::build::origin();
        let data = plots!(l1, l2, l3, l4, l5, l6, l7, m);

        let p = simple_fmt!(data, "hay", "x", "y");

        let mut file =
            std::fs::File::create(path.join("colfind").join(format!("grow_{}.svg", n))).unwrap();

        p.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }
}

fn layout(path: &Path) {
    std::fs::create_dir_all(path.join("layout")).unwrap();

    for grow in [0.2, 2.0] {
        for size in [8, 128, 256] {
            let res1 = layout::bench(layout::Layout::Default, grow, size);
            let res2 = layout::bench(layout::Layout::Direct, grow, size);
            let res3 = layout::bench(layout::Layout::Indirect, grow, size);

            let l1 = scatter("c default", res1.iter().map(|&(i, x, _)| (i as i128, x)));
            let l2 = scatter("c direct", res2.iter().map(|&(i, x, _)| (i as i128, x)));
            let l3 = scatter("c indirect", res3.iter().map(|&(i, x, _)| (i as i128, x)));

            let l4 = scatter("q default", res1.iter().map(|&(i, _, x)| (i as i128, x)));
            let l5 = scatter("q direct", res2.iter().map(|&(i, _, x)| (i as i128, x)));
            let l6 = scatter("q indirect", res3.iter().map(|&(i, _, x)| (i as i128, x)));

            let m = poloto::build::origin();
            let data = plots!(l1, l2, l3, l4, l5, l6, m);

            let p = simple_fmt!(data, formatm!("grow_{}", grow), "x", "y");

            let mut file = std::fs::File::create(
                path.join("layout")
                    .join(format!("rebal_{}_{}.svg", size, grow)),
            )
            .unwrap();

            p.simple_theme(support::upgrade_write(&mut file)).unwrap();
        }
    }
}

fn parallel(path: &Path) {
    std::fs::create_dir_all(path.join("par")).unwrap();

    let res = par_tuner::bench_par(3.0, Some(512), Some(512));

    let l1 = scatter("rebal", res.iter().map(|&(i, _, x)| (i as i128, x)));
    let l2 = scatter("query", res.iter().map(|&(i, x, _)| (i as i128, x)));

    let m = poloto::build::origin();
    let data = plots!(l1, l2, m);

    let p = simple_fmt!(data, "rebal", "x", "y");

    let mut file = std::fs::File::create(path.join("par").join("par-speedup.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();

    let res = par_tuner::best_seq_fallback_rebal(80_000, 2.0);

    let l1 = scatter("", res.iter().map(|&(i, x)| (i as i128, x)));

    let m = poloto::build::origin();
    let data = plots!(l1, m);

    let p = simple_fmt!(data, "rebal", "x", "y");

    let mut file =
        std::fs::File::create(path.join("par").join("optimal-seq-fallback-rebal.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();

    let res = par_tuner::best_seq_fallback_query(80_000, 2.0);

    let l1 = scatter("", res.iter().map(|&(i, x)| (i as i128, x)));

    let m = poloto::build::origin();
    let data = plots!(l1, m);

    let p = simple_fmt!(data, "query", "x", "y");

    let mut file =
        std::fs::File::create(path.join("par").join("optimal-seq-fallback-query.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}

fn float_vs_integer(path: &Path) {
    let res = float_vs_integer::bench(10_000, 2.0);

    let l1 = scatter("f32", res.iter().map(|(i, r)| (*i as i128, r.float)));
    let l2 = scatter("i32", res.iter().map(|(i, r)| (*i as i128, r.int)));
    let l3 = scatter("i64", res.iter().map(|(i, r)| (*i as i128, r.i64)));
    let l4 = scatter(
        "f32->int",
        res.iter().map(|(i, r)| (*i as i128, r.float_i32)),
    );

    let m = poloto::build::origin();
    let data = plots!(l1, l2, l3, l4, m);

    let p = simple_fmt!(data, "float-int", "x", "y");

    let mut file = std::fs::File::create(path.join("float-int.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}
fn rebal_vs_query(path: &Path) {
    let res = rebal_vs_query::bench(80_000, 2.0);
    let l1 = scatter("tree_r", res.iter().map(|(i, r)| (*i as i128, r.tree.0)));
    let l2 = scatter("tree_q", res.iter().map(|(i, r)| (*i as i128, r.tree.1)));
    let l3 = scatter(
        "nosort_r",
        res.iter().map(|(i, r)| (*i as i128, r.nosort.0)),
    );
    let l4 = scatter(
        "nosort_q",
        res.iter().map(|(i, r)| (*i as i128, r.nosort.1)),
    );

    let m = poloto::build::origin();
    let data = plots!(l1, l2, l3, l4, m);

    let p = simple_fmt!(data, "rebal-vs-query", "x", "y");

    let mut file = std::fs::File::create(path.join("rebal-vs-query.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();

    let l1 = scatter("tree_r", res.iter().map(|(i, r)| (*i as i128, r.tree.0)));
    let l2 = scatter("tree_q", res.iter().map(|(i, r)| (*i as i128, r.tree.1)));

    let l3 = scatter(
        "par_tree_r",
        res.iter().map(|(i, r)| (*i as i128, r.par_tree.0)),
    );
    let l4 = scatter(
        "par_tree_q",
        res.iter().map(|(i, r)| (*i as i128, r.par_tree.1)),
    );
    let m = poloto::build::origin();

    let data = plots!(l1, l2, l3, l4, m);

    let p = simple_fmt!(data, "par-rebal-vs-query", "x", "y");

    let mut file = std::fs::File::create(path.join("par-rebal-vs-query.svg")).unwrap();

    p.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}
