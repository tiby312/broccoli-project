use super::*;

const NO_SORT_MAX: usize = 8000;

pub fn handle_bench(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "construction_query_bench",
        title: "Construction vs Query abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..20_000)
            .step_by(80)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots,false))),
        stop_values: &[
            ("nosort_contr", NO_SORT_MAX as f32),
            ("nosort_query", NO_SORT_MAX as f32),
            ("nosort_par_contr", NO_SORT_MAX as f32),
            ("nosort_par_query", NO_SORT_MAX as f32),
        ],
    });
}


#[derive(Debug,Serialize)]
struct Record {
    brocc_contr: f32,
    brocc_query: f32,
    brocc_par_contr:f32,
    brocc_par_query:f32,
    nosort_contr:f32,
    nosort_query:f32,
    nosort_par_contr:f32,
    nosort_par_query:f32,
}

impl Record {
    pub fn new(grow: f64,num_bots: usize,  do_all: bool) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

        let bench = {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let (mut tree, t1) = bench_closure_ret(|| broccoli::new(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_as();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_as();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1 as f32, t2 as f32)
        };

        let bench_par = {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let (mut tree, t1) = bench_closure_ret(|| broccoli::new_par(RayonJoin, &mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_as();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_as();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1 as f32, t2 as f32)
        };

        let nosort = if do_all || num_bots <= NO_SORT_MAX {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let (mut tree, t1) = bench_closure_ret(|| NotSorted::new(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_as();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_as();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1 as f32, t2 as f32)
        } else {
            (0.0, 0.0)
        };

        let nosort_par = if do_all || num_bots <= NO_SORT_MAX {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let (mut tree, t1) = bench_closure_ret(|| NotSorted::new_par(RayonJoin, &mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_as();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_as();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1 as f32, t2 as f32)
        } else {
            (0.0, 0.0)
        };

        Record {
            brocc_contr:bench.0,
            brocc_query:bench.1,
            brocc_par_contr:bench_par.0,
            brocc_par_query:bench_par.1,
            nosort_contr:nosort.0,
            nosort_query:nosort.1,
            nosort_par_contr:nosort_par.0,
            nosort_par_query:nosort_par.1,
        }
    }
}
