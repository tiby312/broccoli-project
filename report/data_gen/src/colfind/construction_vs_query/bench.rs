use super::*;

const NO_SORT_MAX: usize = 8000;
const NO_SORT_PAR_MAX: usize = 15000;

pub fn handle_bench(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "construction_query_bench",
        title: &format!("Bench of construction vs query abspiral(x,{})",DEFAULT_GROW),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0, 20_000)
            .map(|num_bots| (num_bots as f64, Record::new(DEFAULT_GROW, num_bots, false))),
        stop_values: &[
            ("nosort_contr", NO_SORT_PAR_MAX as f64),
            ("nosort_query", NO_SORT_MAX as f64),
            ("nosort_par_contr", NO_SORT_PAR_MAX as f64),
            ("nosort_par_query", NO_SORT_PAR_MAX as f64),
        ],
    });
}

#[derive(Debug, Serialize)]
struct Record {
    brocc_contr: f64,
    brocc_query: f64,
    brocc_par_contr: f64,
    brocc_par_query: f64,
    nosort_contr: f64,
    nosort_query: f64,
    nosort_par_contr: f64,
    nosort_par_query: f64,
}

impl Record {
    pub fn new(grow: f64, num_bots: usize, do_all: bool) -> Record {
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
            (t1 as f64, t2 as f64)
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
            (t1 as f64, t2 as f64)
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
            (t1 as f64, t2 as f64)
        } else {
            (0.0, 0.0)
        };

        let nosort_par = if do_all || num_bots <= NO_SORT_PAR_MAX {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let (mut tree, t1) = bench_closure_ret(|| NotSorted::new_par(RayonJoin, &mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_as();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_as();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1 as f64, t2 as f64)
        } else {
            (0.0, 0.0)
        };

        Record {
            brocc_contr: bench.0,
            brocc_query: bench.1,
            brocc_par_contr: bench_par.0,
            brocc_par_query: bench_par.1,
            nosort_contr: nosort.0,
            nosort_query: nosort.1,
            nosort_par_contr: nosort_par.0,
            nosort_par_query: nosort_par.1,
        }
    }
}
