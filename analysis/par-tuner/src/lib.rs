use broccoli_rayon::build::RayonBuildPar;
use support::prelude::*;

fn single<T: ColfindHandler>(
    bots: &mut [T],
    c_num_seq_fallback: Option<usize>,
    q_num_seq_fallback: Option<usize>,
) -> (f64, f64)
where
    T: Send,
    T::Num: Send,
{
    let (tree, cseq) = bench_closure_ret(|| broccoli::Tree::new(bots));
    black_box(tree);

    let sss = if let Some(c) = c_num_seq_fallback {
        c
    } else {
        broccoli_rayon::build::SEQ_FALLBACK_DEFAULT
    };
    let num_level = broccoli::tree::num_level::default(bots.len());
    let (mut tree, cpar) =
        bench_closure_ret(|| broccoli::Tree::par_new_ext(bots, num_level, sss).0);

    let cspeedup = cseq as f64 / cpar as f64;

    let qseq = bench_closure(|| {
        tree.find_colliding_pairs(T::handle);
    });

    let ccc = if let Some(c) = q_num_seq_fallback {
        c
    } else {
        broccoli_rayon::query::colfind::SEQ_FALLBACK_DEFAULT
    };

    let qpar = bench_closure(|| {
        tree.par_find_colliding_pairs_ext(ccc, T::handle);
    });

    let qspeedup = qseq as f64 / qpar as f64;

    (cspeedup, qspeedup)
}

pub fn bench_par(emp: &mut Html) -> std::fmt::Result {
    let grow = 3.0;
    let description = formatdoc! {r#"
            x speed up of parallel versions.
            `abspiral(n,{grow})`
        "#};

    let res = bench_par_inner(grow, None, None);

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
    )
}
#[inline(never)]
fn bench_par_inner(
    grow: f64,
    c_num_seq_fallback: Option<usize>,
    q_num_seq_fallback: Option<usize>,
) -> Vec<(i128, f64, f64)> {
    let mn = 1_000_000;

    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(mn).collect();

    let mut plots = vec![];

    for i in (0..mn).step_by(1000).skip(1) {
        let bots = &mut all[0..i];

        let (j, k) = single(bots, c_num_seq_fallback, q_num_seq_fallback);

        plots.push((i as i128, j, k));
    }
    plots
}

pub fn best_seq_fallback_rebal(emp: &mut Html) -> std::fmt::Result {
    let num = 80_000;
    let grow = 2.0;
    let description = formatdoc! {r#"
            x speedup of different seq-fallback values during construction
            `abspiral({num},{grow})`
        "#};

    let res = best_seq_fallback_rebal_inner(num, grow);
    let l1 = res.iter().cloned_plot().scatter("");

    let m = poloto::build::origin();

    emp.write_graph(
        Some("par"),
        "optimal-seq-fallback-rebal",
        "num elements",
        "x speedup over sequential",
        l1.chain(m),
        &description,
    )
}
pub fn best_seq_fallback_rebal_inner(num: usize, grow: f64) -> Vec<(i128, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (000..20_000)
        .step_by(10)
        .map(|r| {
            let (a, _) = single(&mut all, Some(r), None);
            (r as i128, a as f64)
        })
        .collect()
}

pub fn best_seq_fallback_query(emp: &mut Html) -> std::fmt::Result {
    let num = 80_000;
    let grow = 2.0;
    let description = formatdoc! {r#"
            x speedup of different seq-fallback values during query
            `abspiral({num},{grow})`
        "#};

    let res = best_seq_fallback_query_inner(num, grow);

    let l1 = res.iter().cloned_plot().scatter("");

    let m = poloto::build::origin();

    emp.write_graph(
        Some("par"),
        "optimal-seq-fallback-query",
        "num elements",
        "x speedup over sequential",
        l1.chain(m),
        &description,
    )
}
fn best_seq_fallback_query_inner(num: usize, grow: f64) -> Vec<(i128, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (000..20_000)
        .step_by(10)
        .map(|a| {
            let (_, b) = single(&mut all, None, Some(a));
            (a as i128, b as f64)
        })
        .collect()
}
