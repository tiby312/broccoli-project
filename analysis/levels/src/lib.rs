use support::datanum::DnumManager;
use support::prelude::queries::colfind::build::CollVis;
use support::prelude::queries::colfind::AccNodeHandler;
use support::prelude::*;

use self::levelcounter::LevelCounter;
use self::leveltimer::LevelTimer;

mod levelcounter;
mod leveltimer;
mod splitter;
// use broccoli::queries::colfind::QueryArgs;

struct Res<X> {
    pub rebal: Vec<X>,
    pub query: Vec<X>,
}

pub fn theory(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    let num = 5_000;
    let description = formatdoc! {r#"
        Comparison of construction of different levels for `abspiral({num},grow)`
    "#};

    let res = theory_inner(man, num, 0.2, 2.0);

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
        "number comparisons",
        poloto::build::plots_dyn(rebals),
        &description,
    )?;

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
        "number of comparisons",
        poloto::build::plots_dyn(queries),
        &description,
    )
}
pub fn bench(emp: &mut Html) -> std::fmt::Result {
    let num = 5_000;
    let description = formatdoc! {r#"
            Comparison of construction of different levels for `abspiral({num},grow)`
        "#};

    let res = bench_inner(num, 0.2, 2.0);

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
        poloto::build::plots_dyn(rebals),
        &description,
    )?;

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
        poloto::build::plots_dyn(queries),
        &description,
    )
}
fn bench_inner(num: usize, start_grow: f64, end_grow: f64) -> Vec<(f64, Res<f64>)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
            let res = gen(&mut all);
            (grow, res)
        })
        .collect()
}

use broccoli::tree::build::DefaultSorter;
use broccoli::tree::build::TreeBuildVisitor;
use broccoli::tree::num_level;

fn theory_inner(
    man: &mut DnumManager,
    num: usize,
    start_grow: f64,
    end_grow: f64,
) -> Vec<(f64, Res<i128>)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist_datanum(man, grow)
                .map(|x| Dummy(x, 0u32))
                .take(num)
                .collect();
            let res = gen_theory(man, &mut all);
            (grow, res)
        })
        .collect()
}

fn gen_theory<T: ColfindHandler>(man: &mut DnumManager, bots: &mut [T]) -> Res<i128> {
    man.reset_counter();

    let mut levelc = LevelCounter::new(man, 0, vec![]);
    let mut tree = {
        let num_level = num_level::default(bots.len());
        let num_nodes = num_level::num_nodes(num_level);
        let mut nodes = Vec::with_capacity(num_nodes);

        crate::splitter::build::recurse_seq_splitter(
            TreeBuildVisitor::new(num_level, bots),
            &mut levelc,
            &mut DefaultSorter,
            &mut nodes,
        );

        assert_eq!(num_nodes, nodes.len());

        Tree::from_nodes(nodes)
    };

    // let (mut tree, levelc) =
    //     Tree::from_build_args(bots, BuildArgs::new(len), );

    let c1 = levelc
        .into_levels()
        .into_iter()
        .map(|x| x as i128)
        .collect();

    man.reset_counter();

    let mut levelc2 = LevelCounter::new(man, 0, vec![]);
    {
        crate::splitter::query::colfind::recurse_seq_splitter(
            CollVis::new(tree.vistr_mut()),
            &mut levelc2,
            &mut AccNodeHandler::new(T::handle),
        );
    }

    let c2 = levelc2
        .into_levels()
        .into_iter()
        .map(|x| x as i128)
        .collect();

    Res {
        rebal: c1,
        query: c2,
    }
}

fn gen<T: ColfindHandler>(bots: &mut [T]) -> Res<f64> {
    let mut times1 = LevelTimer::new(0, vec![]);
    let mut tree = {
        let num_level = num_level::default(bots.len());
        let num_nodes = num_level::num_nodes(num_level);
        let mut nodes = Vec::with_capacity(num_nodes);

        crate::splitter::build::recurse_seq_splitter(
            TreeBuildVisitor::new(num_level, bots),
            &mut times1,
            &mut DefaultSorter,
            &mut nodes,
        );

        assert_eq!(num_nodes, nodes.len());

        Tree::from_nodes(nodes)
    };

    let c1 = times1.into_levels().into_iter().map(|x| x as f64).collect();

    let mut times2 = LevelTimer::new(0, vec![]);
    {
        crate::splitter::query::colfind::recurse_seq_splitter(
            CollVis::new(tree.vistr_mut()),
            &mut times2,
            &mut AccNodeHandler::new(T::handle),
        );
    }

    let c2 = times2.into_levels().into_iter().map(|x| x as f64).collect();

    let t = Res {
        rebal: c1,
        query: c2,
    };

    assert_eq!(t.rebal.len(), t.query.len());
    assert_eq!(t.rebal.len(), t.query.len());
    t
}
