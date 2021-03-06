use super::*;

#[derive(Debug, Serialize)]
struct Record {
    brocc_constr: f64,
    brocc_query: f64,
    nosort_constr: f64,
    nosort_query: f64,
}
impl Record {
    fn new(grow: f64, num_bots: usize) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

        let theory = datanum::datanum_test2(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));

            let mut tree = broccoli::new(&mut bots);

            let count = maker.count();

            tree.find_colliding_pairs_mut(|a, b| {
                let aa = vec2(a.get().x.start.0, a.get().y.start.0).inner_as();
                let bb = vec2(b.get().x.start.0, b.get().y.start.0).inner_as();
                repel(aa, bb, a.unpack_inner(), b.unpack_inner());
            });

            let count2 = maker.count();
            (count as f64, count2 as f64)
        });

        let nosort_theory = datanum::datanum_test2(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));

            let mut tree = NotSorted::new(&mut bots);

            let count = maker.count();

            tree.find_colliding_pairs_mut(|a, b| {
                let aa = vec2(a.get().x.start.0, a.get().y.start.0).inner_as();
                let bb = vec2(b.get().x.start.0, b.get().y.start.0).inner_as();
                repel(aa, bb, a.unpack_inner(), b.unpack_inner());
            });

            let count2 = maker.count();
            (count as f64, count2 as f64)
        });

        Record {
            brocc_constr: theory.0,
            brocc_query: theory.1,
            nosort_constr: nosort_theory.0,
            nosort_query: nosort_theory.1,
        }
    }
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "construction_vs_query_theory_0.2",
        title: "Complexity of construction vs query with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0, 6_000).map(|num_bots| (num_bots as f64, Record::new(0.2, num_bots))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "construction_vs_query_theory_4.0",
        title: "Complexity of construction vs query with abspiral(x,4.0)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0, 6_000).map(|num_bots| (num_bots as f64, Record::new(4.0, num_bots))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "construction_vs_query_theory_grow",
        title: "Complexity of construction vs query with abspiral(40_000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots: grow_iter(0.1, 1.0).map(|g| (g as f64, Record::new(g, 40_000))),
        stop_values: &[],
    });
}
