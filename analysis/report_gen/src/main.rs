use support::prelude::*;

use poloto::build::scatter;
use poloto::prelude::*;

/*
[analysis/report_gen/src/main.rs:8] colfind::bench_one(30_000, 2.0) = Record {
    brocc: 0.004039539,
    brocc_par: 0.003442385,
    sweep: 0.0,
    sweep_par: 0.0,
    naive: 0.0,
    nosort_par: 0.0,
    nosort: 0.0,
}
*/

fn main() {
    let a = datanum::new_session();

    dbg!(colfind::bench_one(30_000, 2.0));
}

fn report() {
    {
        let res = par_tuner::bench_par(3.0, Some(512), Some(512));

        let mut file = std::fs::File::create("par-tuner.svg").unwrap();

        let l1 = scatter("rebal", res.iter().map(|&(i, _, x)| (i as i128, x)));
        let l2 = scatter("query", res.iter().map(|&(i, x, _)| (i as i128, x)));

        let m = poloto::build::origin();
        let data = plots!(l1, l2, m);

        let p = simple_fmt!(data, "rebal", "x", "y");

        p.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }

    {
        std::fs::create_dir_all("layout").unwrap();

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

                let mut file =
                    std::fs::File::create(format!("layout/rebal_{}_{}.svg", size, grow)).unwrap();

                p.simple_theme(support::upgrade_write(&mut file)).unwrap();
            }
        }
    }

    {
        std::fs::create_dir_all("colfind").unwrap();

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

            let mut file = std::fs::File::create(format!("colfind/n_{}.svg", grow)).unwrap();

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

            let mut file = std::fs::File::create(format!("colfind/grow_{}.svg", n)).unwrap();

            p.simple_theme(&mut support::upgrade_write(&mut file))
                .unwrap();
        }
    }
}
