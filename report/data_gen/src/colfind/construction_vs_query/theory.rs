use super::*;

#[derive(Debug,Serialize)]
struct RecordTheory {
    brocc_constr: f32,
    brocc_query:f32,
    nosort_constr: f32,
    nosort_query:f32
}
impl RecordTheory {
    fn new(grow: f64,num_bots: usize) -> RecordTheory {
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
            (count as f32, count2 as f32)
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
            (count as f32, count2 as f32)
        });

        RecordTheory {
            brocc_constr:theory.0,
            brocc_query:theory.1,
            nosort_constr:nosort_theory.0,
            nosort_query:nosort_theory.1
        }
    }
}

/*
fn plot_inner<I: Iterator<Item = (f32, RecordTheory)>>(
    fg: &mut FigureBuilder,
    it: I,
    filename: &str,
    title: &str,
    xname: &str,
    yname: &str,
) {
    let rects: Vec<_> = it.collect();

    let y1 = rects.iter().map(|a| [a.0, a.1.theory.0]);
    let y2 = rects.iter().map(|a| [a.0, a.1.theory.1]);
    let y3 = rects.iter().map(|a| [a.0, a.1.nosort_theory.0]);
    let y4 = rects.iter().map(|a| [a.0, a.1.nosort_theory.1]);

    let mut p = plotato::plot(title, xname, yname);
    p.line("construction", y1);
    p.line("query", y2);
    p.line("nosort_cons", y3);
    p.line("nosort_query", y4);

    fg.finish_plot(p, filename);
}
*/

pub fn handle_theory(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "construction_vs_query_theory_0.2",
        title: "Construction vs Query with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: (0usize..6_000)
            .step_by(20)
            .map(|num_bots| (num_bots as f32, RecordTheory::new(0.2, num_bots))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "construction_vs_query_theory_4.0",
        title: "Construction vs Query with abspiral(x,4.0)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: (0usize..6_000)
            .step_by(20)
            .map(|num_bots| (num_bots as f32, RecordTheory::new(4.0, num_bots))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "construction_vs_query_theory_grow",
        title: "Construction vs Query with abspiral(40_000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots: abspiral_grow_iter2(0.1, 1.0, 0.005)
            .map(|g| (g as f32, RecordTheory::new(g, 40_000))),
        stop_values: &[],
    });


}
