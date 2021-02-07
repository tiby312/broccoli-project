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
    /*
    fn draw(records: &[Record], fg: &mut Figure) {
        
    
        let k = fg
            .axes2d()
            .set_title(
                &"Checked vs Unchecked binning indexing with abspiral(x,1.0)".to_string(),
                &[],
            )
            .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
            .set_x_label("Number of Objects", &[])
            .set_y_label("Time in Seconds", &[]);

        let x = records.iter().map(|a| a.num_bots);
        for index in 0..4 {
            let y = records.iter().map(|a| a.arr[index]);
            k.lines(
                x.clone(),
                y,
                &[Caption(NAMES[index]), Color(COLS[index]), LineWidth(2.0)],
            );
        }
    
    }
    */
}

pub fn handle(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "checked_vs_unchecked_binning",
        title: "Checked vs Unchecked binning indexing with abspiral(x,1.0)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..40_000)
            .step_by(500)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots))),
        stop_values: &[],
    });


}
