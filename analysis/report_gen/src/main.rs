use support::prelude::*;

fn main() {
    {
        let res = par_tuner::bench_par(3.0, Some(512), Some(512));

        let mut file = std::fs::File::create("par-tuner.svg").unwrap();

        use poloto::build::scatter;
        use poloto::prelude::*;
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

                use poloto::build::scatter;
                use poloto::prelude::*;
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
}
