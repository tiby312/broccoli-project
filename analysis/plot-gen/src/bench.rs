use super::*;

pub fn bench(emp: &mut impl GraphEmplace) {
    {
        let num = 30_000;
        let l = broccoli::tree::BuildArgs::new(num);

        let res = best_height::bench(num, 3, l.num_level + 4, 2.0);
        let l1 = res.iter().map(|&(i, r)| (i, r)).cloned_plot().scatter("");

        let m = poloto::build::markers([], [0.0]);

        let description=r##"
hello
=====

* alpha
* beta

# Heading!!!

This is a *Serious* description!!!

~How are you doing????

[example_link](www.google.com)
        "##;
        emp.write_graph(None, "best-height", "height", "time", l1.chain(m),description);
    }

    for grow in [2.0] {
        let res = colfind::bench(60_000, grow, 10000, 20000);
        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.brocc))
            .cloned_plot()
            .scatter("brocc");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.brocc_par))
            .cloned_plot()
            .scatter("brocc_par");
        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.nosort))
            .cloned_plot()
            .scatter("nosort");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.nosort_par))
            .cloned_plot()
            .scatter("nosort_par");
        let l5 = res
            .iter()
            .map(|(i, r)| (i, r.sweep))
            .cloned_plot()
            .scatter("sweep");
        let l6 = res
            .iter()
            .map(|(i, r)| (i, r.sweep_par))
            .cloned_plot()
            .scatter("sweep_par");
        let l7 = res
            .iter()
            .map(|(i, r)| (i, r.naive))
            .cloned_plot()
            .scatter("naive");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("colfind"),
            &format!("n_{}", grow),
            "x",
            "y",
            plots!(l1, l2, l3, l4, l5, l6, l7, m),
            ""
        );
    }

    for n in [60_000] {
        let res = colfind::bench_grow(n, 0.2, 1.5);

        let p = plots!(
            res.iter()
                .map(|(i, r)| (i, r.brocc))
                .cloned_plot()
                .scatter("brocc"),
            res.iter()
                .map(|(i, r)| (i, r.brocc_par))
                .cloned_plot()
                .scatter("brocc_par"),
            res.iter()
                .map(|(i, r)| (i, r.nosort))
                .cloned_plot()
                .scatter("nosort"),
            res.iter()
                .map(|(i, r)| (i, r.nosort_par))
                .cloned_plot()
                .scatter("nosort_par"),
            res.iter()
                .map(|(i, r)| (i, r.sweep))
                .cloned_plot()
                .scatter("sweep"),
            res.iter()
                .map(|(i, r)| (i, r.sweep_par))
                .cloned_plot()
                .scatter("sweep_par"),
            res.iter()
                .map(|(i, r)| (i, r.naive))
                .cloned_plot()
                .scatter("naive"),
            poloto::build::origin()
        );

        emp.write_graph(Some("colfind"), &format!("grow_{}", n), "x", "y", p,"");
    }


    return;

    for grow in [0.2, 2.0] {
        for size in [8, 128, 256] {
            let res1 = layout::bench(layout::Layout::Default, grow, size);
            let res2 = layout::bench(layout::Layout::Direct, grow, size);
            let res3 = layout::bench(layout::Layout::Indirect, grow, size);

            let p = plots!(
                res1.iter()
                    .map(|(i, x, _)| (i, x))
                    .cloned_plot()
                    .scatter("c default"),
                res2.iter()
                    .map(|(i, x, _)| (i, x))
                    .cloned_plot()
                    .scatter("c direct"),
                res3.iter()
                    .map(|(i, x, _)| (i, x))
                    .cloned_plot()
                    .scatter("c indirect"),
                res1.iter()
                    .map(|(i, _, x)| (i, x))
                    .cloned_plot()
                    .scatter("q default"),
                res2.iter()
                    .map(|(i, _, x)| (i, x))
                    .cloned_plot()
                    .scatter("q direct"),
                res3.iter()
                    .map(|(i, _, x)| (i, x))
                    .cloned_plot()
                    .scatter("q indirect"),
                poloto::build::origin()
            );

            emp.write_graph(
                Some("layout"),
                &format!("rebal_{}_{}", size, grow),
                "x",
                "y",
                p,
                ""
            );
        }
    }

    {
        let res = par_tuner::bench_par(3.0, Some(512), Some(512));

        let p = plots!(
            res.iter()
                .map(|(i, _, x)| (i, x))
                .cloned_plot()
                .scatter("rebal"),
            res.iter()
                .map(|(i, x, _)| (i, x))
                .cloned_plot()
                .scatter("query"),
            poloto::build::origin()
        );

        emp.write_graph(Some("par"), "par-speedup", "x", "y", p,"");
    }

    {
        let res = par_tuner::best_seq_fallback_rebal(80_000, 2.0);
        let l1 = res.iter().cloned_plot().scatter("");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("par"),
            "optimal-seq-fallback-rebal",
            "x",
            "y",
            l1.chain(m),
            ""
        );
    }

    {
        let res = par_tuner::best_seq_fallback_query(80_000, 2.0);

        let l1 = res.iter().cloned_plot().scatter("");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("par"),
            "optimal-seq-fallback-query",
            "x",
            "y",
            l1.chain(m),
            ""
        );
    }

    {
        let res = float_vs_integer::bench(10_000, 2.0);
        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.float))
            .cloned_plot()
            .scatter("f32");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.int))
            .cloned_plot()
            .scatter("i32");
        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.i64))
            .cloned_plot()
            .scatter("i64");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.float_i32))
            .cloned_plot()
            .scatter("f32->int");

        let m = poloto::build::origin();

        emp.write_graph(None, "float-int", "x", "y", plots!(l1, l2, l3, l4, m),"");
    }

    {
        let res = rebal_vs_query::bench(80_000, 2.0);
        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.tree.0))
            .cloned_plot()
            .scatter("tree_r");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.tree.1))
            .cloned_plot()
            .scatter("tree_q");
        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.nosort.0))
            .cloned_plot()
            .scatter("nosort_r");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.nosort.1))
            .cloned_plot()
            .scatter("nosort_q");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("rebal_vs_query"),
            "rebal_vs_query",
            "x",
            "y",
            plots!(l1, l2, l3, l4, m),
            ""
        );

        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.tree.0))
            .cloned_plot()
            .scatter("tree_r");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.tree.1))
            .cloned_plot()
            .scatter("tree_q");

        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.par_tree.0))
            .cloned_plot()
            .scatter("par_tree_r");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.par_tree.1))
            .cloned_plot()
            .scatter("par_tree_q");
        let m = poloto::build::origin();

        emp.write_graph(
            Some("rebal_vs_query"),
            "par-rebal-vs-query",
            "x",
            "y",
            plots!(l1, l2, l3, l4, m),
            ""
        );
    }
}
