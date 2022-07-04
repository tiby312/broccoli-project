use super::*;

use indoc::formatdoc;

pub fn bench(emp: &mut impl GraphEmplace) {
    {
        let grow = 2.0;
        let num = 30_000;
        let description = formatdoc! {r#"
            Optimal height vs heur height for `abspiral({num},{grow})`
        "#};

        let l = broccoli::tree::BuildArgs::new(num);

        let res = best_height::optimal(num, grow);

        let l1 = res
            .iter()
            .map(|(i, r)| (*i, r.optimal_height))
            .cloned_plot()
            .scatter("optimal");
        let l2 = res
            .iter()
            .map(|(i, r)| (*i, r.heur_height))
            .cloned_plot()
            .scatter("heur");

        emp.write_graph(
            Some("height"),
            "heuristic",
            "num elements",
            "time taken (seconds)",
            l1.chain(l2),
            &description,
        );
    }

    {
        let grow = 2.0;
        let num = 30_000;
        let description = formatdoc! {r#"
            Bench time to solve `abspiral({num},{grow})` with 
            different tree heights
        "#};

        let l = broccoli::tree::BuildArgs::new(num);

        let res = best_height::bench(num, 3, l.num_level + 4, grow);
        let l1 = res.iter().map(|&(i, r)| (i, r)).cloned_plot().scatter("");

        let m = poloto::build::markers([], [0.0]);

        emp.write_graph(
            Some("height"),
            "best-height",
            "tree height",
            "time taken (seconds)",
            l1.chain(m),
            &description,
        );
    }

    {
        let num = 10_000;
        let grow = 1.0;
        let num_iter = 2;

        let description = formatdoc! {r#"
            Query vs Cached Query with {num_iter} iterations of `abspiral(num,{grow})`.
        "#};
        let res = cached_pairs::bench(num, grow, num_iter);

        let a = res
            .iter()
            .map(|(x, y)| (*x, y.bench))
            .cloned_plot()
            .scatter("no cache");
        let b = res
            .iter()
            .map(|(x, y)| (*x, y.collect))
            .cloned_plot()
            .scatter("cached");

        emp.write_graph(
            None,
            "collect",
            "num elements",
            "time taken (seconds)",
            a.chain(b),
            &description,
        );
    }

    {
        let num = 5_000;
        let description = formatdoc! {r#"
            Comparison of construction of different levels for `abspiral({num},grow)`
        "#};

        let res = levels::bench(num, 0.2, 2.0);

        let num_level = res[0].1.rebal.len();

        let rebals: Vec<_> = (0..num_level)
            .map(|i| {
                let k = res
                    .iter()
                    .map(move |(x, y)| (*x, y.rebal[i]))
                    .cloned_plot()
                    .line_fill(formatm!("level {}", i));
                k
            })
            .collect();

        emp.write_graph(
            Some("levels"),
            "rebal",
            "grow",
            "time taken (seconds)",
            poloto::build::plots_dyn::<f64, f64, _>(rebals),
            &description,
        );

        let description = formatdoc! {r#"
            Comparison of querying for different levels for `abspiral({num},grow)`
        "#};

        let queries: Vec<_> = (0..num_level)
            .map(|i| {
                let k = res
                    .iter()
                    .map(move |(x, y)| (*x, y.query[i]))
                    .cloned_plot()
                    .line_fill(formatm!("level {}", i));
                k
            })
            .collect();

        emp.write_graph(
            Some("levels"),
            "query",
            "grow",
            "time taken (seconds)",
            poloto::build::plots_dyn::<f64, f64, _>(queries),
            &description,
        );
    }

    {
        let grow = 2.0;
        let description = formatdoc! {r#"
            Comparison of bench times using different number types as problem
            size increases. `abspiral(n,{grow})`
        "#};

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

        emp.write_graph(
            None,
            "float-int",
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, m),
            &description,
        );
    }

    for grow in [2.0] {
        let description = formatdoc! {r#"
            Comparison of bench times of different collision finding strategies. 
            `abspiral(n,{grow})`
        "#};

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
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, l5, l6, l7, m),
            &description,
        );
    }

    for n in [60_000] {
        let description = formatdoc! {r#"
            Comparison of bench times of different collision finding strategies. 
            `abspiral({n},x)`
        "#};

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
            poloto::build::markers([], [0.0])
        );

        emp.write_graph(
            Some("colfind"),
            &format!("grow_{}", n),
            "grow",
            "time taken (seconds)",
            p,
            &description,
        );
    }

    for grow in [0.2, 2.0] {
        for size in [8, 128, 256] {
            let description = formatdoc! {r#"
                Comparison of bench times with elements with {size} bytes. 
                `abspiral(n,{grow})`
            "#};

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
                "num elements",
                "time taken (seconds)",
                p,
                &description,
            );
        }
    }

    {
        let grow = 3.0;
        let description = formatdoc! {r#"
            x speed up of parallel versions.
            `abspiral(n,{grow})`
        "#};

        let res = par_tuner::bench_par(grow, None, None);

        let p = plots!(
            res.iter()
                .map(|(i, x, _)| (i, x))
                .cloned_plot()
                .scatter("rebal"),
            res.iter()
                .map(|(i, _, x)| (i, x))
                .cloned_plot()
                .scatter("query"),
            poloto::build::origin()
        );

        emp.write_graph(
            Some("par"),
            "par-speedup",
            "num elements",
            "x speedup over sequential",
            p,
            &description,
        );
    }

    {
        let num = 80_000;
        let grow = 2.0;
        let description = formatdoc! {r#"
            x speedup of different seq-fallback values during construction
            `abspiral({num},{grow})`
        "#};

        let res = par_tuner::best_seq_fallback_rebal(num, grow);
        let l1 = res.iter().cloned_plot().scatter("");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("par"),
            "optimal-seq-fallback-rebal",
            "num elements",
            "x speedup over sequential",
            l1.chain(m),
            &description,
        );
    }

    {
        let num = 80_000;
        let grow = 2.0;
        let description = formatdoc! {r#"
            x speedup of different seq-fallback values during query
            `abspiral({num},{grow})`
        "#};

        let res = par_tuner::best_seq_fallback_query(num, grow);

        let l1 = res.iter().cloned_plot().scatter("");

        let m = poloto::build::origin();

        emp.write_graph(
            Some("par"),
            "optimal-seq-fallback-query",
            "num elements",
            "x speedup over sequential",
            l1.chain(m),
            &description,
        );
    }

    {
        let num = 80_000;
        let grow = 2.0;
        let description = formatdoc! {r#"
            comparison of construction vs query
            `abspiral({num},{grow})`
        "#};

        let res = rebal_vs_query::bench(num, grow);
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
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, m),
            &description,
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
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, m),
            &description,
        );
    }
}
