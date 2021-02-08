use crate::inner_prelude::*;


#[derive(Debug,Serialize)]
struct Record {
    checked_par:f32,
    not_checked_par:f32,
    checked:f32,
    not_checked:f32
}

impl Record {
    fn new(grow:f64,num_bots:usize)->Record{
        let mut bot_inner:Vec<_>=(0usize..num_bots).collect();
        let checked_par={
            let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let tree = TreeBuilder::new(&mut scene)
                    .with_bin_strat(BinStrat::Checked)
                    .build_par(RayonJoin);

                black_box(tree);
            }) as f32
        };
        let not_checked_par={
            let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let tree = TreeBuilder::new(&mut scene)
                    .with_bin_strat(BinStrat::NotChecked)
                    .build_par(RayonJoin);

                black_box(tree);
            }) as f32
        };
        let checked={
            let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let tree = TreeBuilder::new(&mut scene)
                    .with_bin_strat(BinStrat::Checked)
                    .build_seq();

                black_box(tree);
            }) as f32
        };
        let not_checked={
            let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let tree = TreeBuilder::new(&mut scene)
                    .with_bin_strat(BinStrat::NotChecked)
                    .build_seq();

                black_box(tree);
            }) as f32
        };
        Record{
            checked_par,
            not_checked_par,
            checked,
            not_checked,
        }
    }
}

pub fn handle(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "checked_vs_unchecked_binning",
        title: "Bench of checked vs unchecked binning with abspiral(x,1.0)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..40_000)
            .step_by(200)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots))),
        stop_values: &[],
    });


}
