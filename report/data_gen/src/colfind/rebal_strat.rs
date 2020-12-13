use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;
#[derive(Copy, Clone)]
pub struct Bot {
    _num: usize,
    pos: Vec2<i32>,
}

fn test1(scene: &mut [BBox<NotNan<f32>, &mut isize>]) -> f64 {
    bench_closure(|| {
        let tree = TreeBuilder::new(scene)
            .with_bin_strat(BinStrat::Checked)
            .build_par();

        black_box(tree);
    })
}

fn test2(scene: &mut [BBox<NotNan<f32>, &mut isize>]) -> f64 {
    bench_closure(|| {
        let tree = TreeBuilder::new(scene)
            .with_bin_strat(BinStrat::NotChecked)
            .build_par();

        black_box(tree);
    })
}

fn test3(scene: &mut [BBox<NotNan<f32>, &mut isize>]) -> f64 {
    bench_closure(|| {
        let tree = TreeBuilder::new(scene)
            .with_bin_strat(BinStrat::Checked)
            .build_seq();

        black_box(tree);
    })
}

fn test4(scene: &mut [BBox<NotNan<f32>, &mut isize>]) -> f64 {
    bench_closure(|| {
        let tree = TreeBuilder::new(scene)
            .with_bin_strat(BinStrat::NotChecked)
            .build_seq();

        black_box(tree);
    })
}

pub fn handle(fb: &mut FigureBuilder) {
    handle_num_bots(fb, 0.2);
}

#[derive(Debug)]
struct Record {
    num_bots: usize,
    arr: [f64; 4],
}
impl Record {
    fn draw(records: &[Record], fg: &mut Figure) {
        const NAMES: &[&str] = &[
            "RebalStrat Checked Par",
            "RebalStrat Not Checked Par",
            "RebalStrat Checked Seq",
            "RebalStrat Not Checked Seq",
        ];
        {
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
    }
}

fn handle_num_bots(fb: &mut FigureBuilder, grow: f64) {
    let mut rects = Vec::new();

    for num_bots in (0..700_000).step_by(5000) {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let arr = [
            test1(&mut distribute(grow, &mut bot_inner, |a| a.to_f32n())),
            test2(&mut distribute(grow, &mut bot_inner, |a| a.to_f32n())),
            test3(&mut distribute(grow, &mut bot_inner, |a| a.to_f32n())),
            test4(&mut distribute(grow, &mut bot_inner, |a| a.to_f32n())),
        ];

        let r = Record { num_bots, arr };
        rects.push(r);
    }

    let mut fg = fb.build("checked_vs_unchecked_binning");

    Record::draw(&rects, &mut fg);

    fb.finish(fg);
}
