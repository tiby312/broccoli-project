use support::prelude::*;
use crate::broccoli::build::TreeEmbryo;
pub fn bench(emp: &mut Html) -> std::fmt::Result {
    let grow = 2.0;
    let num = 30_000;
    let description = formatdoc! {r#"
            Bench time to solve `abspiral({num},{grow})` with 
            different tree heights
        "#};

    let num_level = broccoli::num_level::default(num);

    let res = bench_inner(num, 3, num_level + 4, grow);
    let l1 = plot("").scatter(pcloned(res.iter().map(|&(i, r)| (i, r))));

    let m = poloto::build::markers([], [0.0]);

    emp.write_graph(
        Some("height"),
        "best-height",
        "tree height",
        "time taken (seconds)",
        plots!(l1, m),
        &description,
    )
}
#[inline(never)]
fn bench_inner(max: usize, min_height: usize, max_height: usize, grow: f64) -> Vec<(i128, f64)> {
    assert!(min_height >= 1);
    assert!(max_height >= min_height);

    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (min_height..max_height)
        .flat_map(|x| std::iter::repeat(x).take(5))
        .map(move |height| {
            let f = new_bench_record(&mut all, height);
            (height as i128, f)
        })
        .collect()
}

pub fn theory(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    let grow = 2.0;
    let num = 30_000;
    let description = formatdoc! {r#"
        theory time to solve `abspiral({num},{grow})` with 
        different tree heights
    "#};

    let num_level = broccoli::num_level::default(num);

    let res = theory_inner(man, num, 3, num_level + 4, grow);
    let l1 = plot("").scatter(pcloned(res.iter().map(|&(i, r)| (i, r))));

    let m = poloto::build::markers([], [0]);

    emp.write_graph(
        Some("height"),
        "best-height",
        "tree height",
        "num comparison",
        plots!(l1, m),
        &description,
    )
}

#[inline(never)]
fn theory_inner(
    man: &mut DnumManager,
    max: usize,
    min_height: usize,
    max_height: usize,
    grow: f64,
) -> Vec<(i128, i128)> {
    assert!(min_height >= 1);
    assert!(max_height >= min_height);

    let mut all: Vec<_> = dist::dist_datanum(man, grow)
        .map(|x| Dummy(x, 0u32))
        .take(max)
        .collect();

    (min_height..max_height)
        .map(move |height| {
            let f = new_theory_record(man, &mut all, height);
            (height as i128, f as i128)
        })
        .collect()
}

struct Res {
    pub optimal_height: i128,
    pub heur_height: i128,
}

pub fn optimal(emp: &mut Html) -> std::fmt::Result {
    for grow in [1.0, 2.0, 4.0] {
        let num = 40_000;
        let description = formatdoc! {r#"
            Optimal height vs heur height for `abspiral({num},{grow})`
        "#};

        let res = optimal_inner(num, grow);

        let l1 = plot("optimal")
            .scatter(pcloned(res.iter().map(|(i, r)| (*i, r.optimal_height))));

        let l2 = plot("heur")
            .scatter(pcloned(res.iter().map(|(i, r)| (*i, r.heur_height))));

        emp.write_graph(
            Some("height"),
            "heuristic",
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2),
            &description,
        )?;
    }
    Ok(())
}

#[inline(never)]
fn optimal_inner(num: usize, grow: f64) -> Vec<(i128, Res)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (0..num)
        .step_by(1000)
        .map(move |n| {
            let bots = &mut all[0..n];

            let optimal_height = (1..20)
                .map(|height| (height, new_bench_record(bots, height)))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;

            let heur_height = broccoli::num_level::default(n);

            (
                n as i128,
                Res {
                    optimal_height: optimal_height as i128,
                    heur_height: heur_height as i128,
                },
            )
        })
        .collect()
}

fn new_theory_record<T: ColfindHandler>(
    man: &mut DnumManager,
    bots: &mut [T],
    height: usize,
) -> usize {
    man.time(|| {
        use broccoli::build::DefaultSorter;
        use broccoli::build::TreeBuildVisitor;

        let (mut e,v)=TreeEmbryo::with_num_level(bots,height);
        e.recurse(v,&mut DefaultSorter);
        let mut tree=e.finish();

        tree.find_colliding_pairs(T::handle);
    })
}

fn new_bench_record<T: ColfindHandler>(bots: &mut [T], height: usize) -> f64 {
    let mut bencher = Bencher;

    bencher.time(|| {
        use broccoli::build::DefaultSorter;
        use broccoli::build::TreeBuildVisitor;


        let (mut e,v)=TreeEmbryo::with_num_level(bots,height);
        e.recurse(v,&mut DefaultSorter);
        let mut tree=e.finish();

        assert_eq!(tree.num_levels(), height);

        tree.find_colliding_pairs(T::handle);
    })
}
