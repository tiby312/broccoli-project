use std::path::Path;

use poloto::build::scatter;
use poloto::prelude::*;
use support::datanum::DnumManager;
use support::prelude::*;

fn main() {
    let mut a = datanum::new_session();
    theory(&mut a, &Path::new("../../target/analysis"));
}

fn theory(man: &mut DnumManager, path: &Path) {
    {
        std::fs::create_dir_all(path.join("colfind")).unwrap();

        for grow in [2.0] {
            let res = colfind::theory(man, 5_000, grow, 1000, 20000);

            let l1 = scatter("brocc", res.iter().map(|(i, r)| (*i as i128, r.brocc)));
            let l2 = scatter("nosort", res.iter().map(|(i, r)| (*i as i128, r.nosort)));
            let l3 = scatter("sweep", res.iter().map(|(i, r)| (*i as i128, r.sweep)));
            let l4 = scatter("naive", res.iter().map(|(i, r)| (*i as i128, r.naive)));

            let m = poloto::build::origin();
            let data = plots!(l1, l2, l3, l4, m);

            let p = simple_fmt!(data, "hay", "x", "y");

            let mut file =
                std::fs::File::create(path.join("colfind").join(format!("theory_n_{}.svg", grow)))
                    .unwrap();

            p.simple_theme(&mut support::upgrade_write(&mut file))
                .unwrap();
        }
    }
    {
        std::fs::create_dir_all(path.join("level")).unwrap();
        let res = levels::theory(man, 20_000, 0.2, 2.0);

        let num_level = res[0].1.rebal.len();

        let data = (0usize..num_level)
            .map(|i| {
                let g = res
                    .iter()
                    .map(move |(grow, levels)| (*grow, levels.rebal[i] as i128));
                poloto::build::line_fill(formatm!("Level {}", i), g)
            })
            .collect();

        let mut file = std::fs::File::create(path.join("level").join("rebal.svg")).unwrap();

        let plot = poloto::simple_fmt!(
            poloto::build::plots_dyn(data).chain(poloto::build::markers([], [0])),
            "rebal",
            "Spiral Grow",
            "Number of Comparisons"
        );

        plot.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }
}

fn report(path: &Path) {
    {
        let res = par_tuner::bench_par(3.0, Some(512), Some(512));

        let mut file = std::fs::File::create(path.join("par-tuner.svg")).unwrap();

        let l1 = scatter("rebal", res.iter().map(|&(i, _, x)| (i as i128, x)));
        let l2 = scatter("query", res.iter().map(|&(i, x, _)| (i as i128, x)));

        let m = poloto::build::origin();
        let data = plots!(l1, l2, m);

        let p = simple_fmt!(data, "rebal", "x", "y");

        p.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }

    {
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

    {
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
                std::fs::File::create(path.join("colfind").join(format!("n_{}.svg", grow)))
                    .unwrap();

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
                std::fs::File::create(path.join("colfind").join(format!("grow_{}.svg", n)))
                    .unwrap();

            p.simple_theme(&mut support::upgrade_write(&mut file))
                .unwrap();
        }
    }
}
