use broccoli_rayon::prelude::*;
use support::prelude::*;

pub fn theory(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    let num = 10_000;
    let grow = 2.0;
    let description = formatdoc! {r#"
        comparison of construction vs query
        `abspiral({num},{grow})`
    "#};

    let res = theory_inner(man, num, grow);
    let p=plots!(
        plot("tree_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.0)))),
        plot("tree_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.1)))),
        plot("nosort_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.nosort.0)))),
        plot("nosort_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.nosort.1)))),
        poloto::build::origin()
    );

    emp.write_graph(
        Some("rebal_vs_query"),
        "par-rebal-vs-query",
        "num elements",
        "number of comparisons",
        p,
        &description,
    )
}

pub fn bench(emp: &mut Html) -> std::fmt::Result {
    let num = 10_000;
    let grow = 2.0;
    let description = formatdoc! {r#"
            comparison of construction vs query
            `abspiral({num},{grow})`
        "#};

    let res = bench_inner(num, grow);

    let p=plots!(
        plot("tree_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.0)))),
        plot("tree_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.1)))),
        plot("nosort_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.nosort.0)))),
        plot("nosort_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.nosort.1)))),
        poloto::build::origin()
    );

    emp.write_graph(
        Some("rebal_vs_query"),
        "rebal_vs_query",
        "num elements",
        "time taken (seconds)",
        p,
        &description,
    )?;


    let p=plots!(
        plot("tree_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.0)))),
        plot("tree_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.tree.1)))),
        plot("par_tree_r").scatter(pcloned(res.iter().map(|(i, r)| (i, r.par_tree.0)))),
        plot("par_tree_q").scatter(pcloned(res.iter().map(|(i, r)| (i, r.par_tree.1)))),
        poloto::build::origin()
    );

    emp.write_graph(
        Some("rebal_vs_query"),
        "par-rebal-vs-query",
        "num elements",
        "time taken (seconds)",
        p,
        &description,
    )
}

#[inline(never)]
fn bench_inner(max: usize, grow: f64) -> Vec<(i128, ParRecord)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .skip(1)
        .map(|a| {
            let bots = &mut all[0..a];
            (a as i128, new_record(bots))
        })
        .collect()
}

#[inline(never)]
fn theory_inner(man: &mut DnumManager, max: usize, grow: f64) -> Vec<(i128, TheoryRecord)> {
    let mut all: Vec<_> = dist::dist_datanum(man, grow)
        .map(|x| Dummy(x, 0u32))
        .take(max)
        .collect();

    (0..max)
        .step_by(100)
        .skip(1)
        .map(|a| {
            let bots = &mut all[0..a];
            (a as i128, new_record_theory(bots, man))
        })
        .collect()
}

#[derive(Debug)]
struct TheoryRecord {
    pub tree: (i128, i128),
    pub nosort: (i128, i128),
}

fn new_record_theory<T: ColfindHandler>(bots: &mut [T], man: &mut DnumManager) -> TheoryRecord {
    let recorder = man;

    let (mut tree, tree1) = recorder.time_ext(|| broccoli::Tree::new(bots));

    let tree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    /*
    let (mut tree,par_notree1) = recorder.time_ext(|| {
        broccoli::NotSortedTree::par_new(bots)
    });

    let par_notree2=recorder.time(||{
        tree.par_find_colliding_pairs(T::handle);
    });
    */

    let (mut tree, notree1) = recorder.time_ext(|| not_sorted::NotSortedTree::new(bots));

    let notree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    TheoryRecord {
        tree: (tree1 as i128, tree2 as i128),
        nosort: (notree1 as i128, notree2 as i128),
    }
}

#[derive(Debug)]
struct ParRecord {
    pub tree: (f64, f64),
    pub par_tree: (f64, f64),
    pub nosort: (f64, f64),
}

fn new_record<T: ColfindHandler>(bots: &mut [T]) -> ParRecord
where
    T: Send,
    T::Num: Send,
{
    let mut recorder = Bencher;
    let (mut tree, par_tree1) = recorder.time_ext(|| {
        broccoli::Tree::par_new(bots)
        //broccoli_rayon::build::par_new2(bots)
    });

    let par_tree2 = recorder.time(|| {
        tree.par_find_colliding_pairs(T::handle);
    });

    let (mut tree, tree1) = recorder.time_ext(|| broccoli::Tree::new(bots));

    let tree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    /*
    let (mut tree,par_notree1) = recorder.time_ext(|| {
        broccoli::NotSortedTree::par_new(bots)
    });

    let par_notree2=recorder.time(||{
        tree.par_find_colliding_pairs(T::handle);
    });
    */

    let (mut tree, notree1) = recorder.time_ext(|| not_sorted::NotSortedTree::new(bots));

    let notree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    ParRecord {
        tree: (tree1, tree2),
        par_tree: (par_tree1, par_tree2),
        nosort: (notree1, notree2),
        //par_nosort:(par_notree1,par_notree2)
    }
}
