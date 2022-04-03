use broccoli::queries::colfind::HandleSorted;

use super::*;

struct MyBuild{
    height_seq_fallback:usize
}
impl<T:Aabb> TreeBuild<T,DefaultSorter> for MyBuild{
    fn height_seq_fallback(&self) -> usize {
        self.height_seq_fallback
    }
    fn sorter(&self) -> DefaultSorter {
        DefaultSorter
    }
}

struct MyQuery<'a,T:Aabb>{
    tree:Tree<'a,T>,
    height_seq_fallback:usize
}

impl<'a,T:Aabb+'a> broccoli::queries::colfind::CollidingPairsBuilder<'a,T,HandleSorted> for MyQuery<'a,T>{
    fn height_seq_fallback(&self)->usize{
        self.height_seq_fallback
    }
    fn colliding_pairs_builder<'b>(&'b mut self) -> queries::colfind::CollVis<'a, 'b, T, HandleSorted> {
        self.tree.colliding_pairs_builder()
    }
}


fn test1(bots: &mut [BBox<f64, &mut isize>]) -> (f64, f64) {
    let (mut tree, construction_time) = bench_closure_ret(|| broccoli::tree::new(bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.colliding_pairs(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}


fn test3(
    bots: &mut [BBox<f64, &mut isize>],
    rebal_height: usize,
    query_height: usize,
) -> (f64, f64) {
    let (tree, construction_time) = bench_closure_ret(|| {
        MyBuild{height_seq_fallback:rebal_height}.build_par(bots)
    });

    let (tree, query_time) = bench_closure_ret(|| {
        let mut tree=MyQuery{tree,height_seq_fallback:query_height};
        tree.colliding_pairs_par(|a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

pub fn handle(fb: &mut FigureBuilder) {
    let num_bots = 20_000;

    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let height=tree::num_level::num_nodes(num_bots);

    let mut rebals = Vec::new();
    for rebal_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (a, _b) = test3(
            &mut distribute(DEFAULT_GROW, &mut bot_inner, |a| a.to_f64n()),
            rebal_height,
            4,
        );
        rebals.push((rebal_height as f64, a as f64));
    }

    let mut queries = Vec::new();
    for query_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (_a, b) = test3(
            &mut distribute(0.2, &mut bot_inner, |a| a.to_f64n()),
            4,
            query_height,
        );
        queries.push((query_height as f64, b as f64));
    }

    let mut seqs = Vec::new();
    for _ in 0..100 {
        let (a, b) = test1(&mut distribute(DEFAULT_GROW, &mut bot_inner, |a| {
            a.to_f64n()
        }));
        seqs.push((a as f64, b as f64));
    }

    let s = format!(
        "Bench of differing parallel switch levels with abspiral(20,000,{})",
        DEFAULT_GROW
    );

    let data = plots!(
        poloto::build::scatter("Rebal Par", rebals.iter().map(|a| [a.0, a.1])),
        poloto::build::scatter("Query Par", queries.iter().map(|a| [a.0, a.1])),
        poloto::build::scatter("Rebal", seqs.iter().map(|a| [height as f64, a.0])),
        poloto::build::scatter("Query", seqs.iter().map(|a| [height as f64, a.1]))
    );

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        data.markers([], [0.0]),
        &s,
        "Height at which to switch to sequential",
        "Time in Seconds"
    );

    fb.finish_plot(
        poloto::disp(|w| plot.render(w)),
        "parallel_height_heuristic",
    );
}
