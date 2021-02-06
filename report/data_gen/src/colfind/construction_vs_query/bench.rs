use super::*;

const NO_SORT_MAX:usize=8000;

pub fn handle_bench(fb: &mut FigureBuilder) {

    handle_bench_inner(
        (0..20_000).step_by(80).map(|n |(n as f32,RecordBench::new(n,0.2,false))),
        fb,
        "construction_query_bench",
        "Construction vs Query",
        "Number of Elements",
        "Time in Seconds"
    );

}
//TODO make this a macro function???
fn handle_bench_inner<I: Iterator<Item = (f32, RecordBench)>>(
    it: I,
    fg: &mut FigureBuilder,
    filename: &str,
    title: &str,
    xname: &str,
    yname: &str,
) {
    let rects: Vec<_> = it.collect();

    let mut plot = plotato::plot(title, xname, yname);

    plot.line(
        "bench constr",
        rects
            .iter()
            .map(|a| [a.0, a.1.bench.0])   
    );
    plot.line(
        "bench query",
        rects
            .iter()
            .map(|a| [a.0, a.1.bench.1])   
    );


    plot.line(
        "bench_par constr",
        rects
            .iter()
            .map(|a| [a.0, a.1.bench_par.0])
    );
    plot.line(
        "bench_par query",
        rects
            .iter()
            .map(|a| [a.0, a.1.bench_par.1])
    );
    
    plot.line(
        "nosort const",
        rects.iter().map(|a| [a.0, a.1.nosort.0]).take_while(|&[x, _]| x <= NO_SORT_MAX as f32),
    );

    plot.line(
        "nosort query",
        rects.iter().map(|a| [a.0, a.1.nosort.1]).take_while(|&[x, _]| x <= NO_SORT_MAX as f32),
    );


    plot.line(
        "nosort_par constr",
        rects.iter().map(|a| [a.0, a.1.nosort_par.0]).take_while(|&[x, _]| x <= NO_SORT_MAX as f32),
    );

    plot.line(
        "nosort_par query",
        rects.iter().map(|a| [a.0, a.1.nosort_par.1]).take_while(|&[x, _]| x <= NO_SORT_MAX as f32),
    );

    fg.finish_plot(plot, filename);
}

/*
fn handle_num_bots_bench_inner(fg: &mut Figure, grow: f64, position: u32) {
    let mut rects: Vec<_> = Vec::new();

    for num_bots in (1..5_000).step_by(20) {
        rects.push(all::handle_bench(num_bots, grow, false));
    }

    let x = rects.iter().map(|a| a.num_bots);

    let y1 = rects.iter().map(|a| a.bench.unwrap().0);
    let y2 = rects.iter().map(|a| a.bench.unwrap().1);
    let y3 = rects.iter().map(|a| a.bench_par.unwrap().0);
    let y4 = rects.iter().map(|a| a.bench_par.unwrap().1);

    let y5 = rects
        .iter()
        .take_while(|a| a.nosort.is_some())
        .map(|a| a.nosort.unwrap().0);
    let y6 = rects
        .iter()
        .take_while(|a| a.nosort.is_some())
        .map(|a| a.nosort.unwrap().1);
    let y7 = rects
        .iter()
        .take_while(|a| a.nosort_par.is_some())
        .map(|a| a.nosort_par.unwrap().0);
    let y8 = rects
        .iter()
        .take_while(|a| a.nosort_par.is_some())
        .map(|a| a.nosort_par.unwrap().1);

    fg.axes2d()
        .set_pos_grid(2, 1, position)
        .set_title(
            &format!("Rebal vs Query Benches with abspiral(x,{})", grow),
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebal Sequential"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color(COLS[1]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color(COLS[2]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color(COLS[3]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color(COLS[4]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color(COLS[5]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color(COLS[6]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color(COLS[7]),
                LineWidth(1.0),
            ],
        )
        .set_x_label("Number of Elements", &[])
        .set_y_label("Time in seconds", &[]);
}

fn handle_grow_bench(fb: &mut FigureBuilder) {
    let num_bots = 50_000;

    let mut rects: Vec<_> = Vec::new();

    for grow in abspiral_grow_iter2(0.2, 0.8, 0.001) {
        rects.push(all::handle_bench(num_bots, grow, true));
    }

    let x = rects.iter().map(|a| a.grow as f64);

    let y1 = rects.iter().map(|a| a.bench.unwrap().0);
    let y2 = rects.iter().map(|a| a.bench.unwrap().1);
    let y3 = rects.iter().map(|a| a.bench_par.unwrap().0);
    let y4 = rects.iter().map(|a| a.bench_par.unwrap().1);

    let y5 = rects.iter().map(|a| a.nosort.unwrap().0);
    let y6 = rects.iter().map(|a| a.nosort.unwrap().1);
    let y7 = rects.iter().map(|a| a.nosort_par.unwrap().0);
    let y8 = rects.iter().map(|a| a.nosort_par.unwrap().1);

    let mut fg = fb.build("construction_vs_query_grow_bench");

    fg.axes2d()
        .set_title("Rebal vs Query Benches with abspiral(80000,x)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebal Sequential"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color(COLS[1]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color(COLS[2]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color(COLS[3]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color(COLS[4]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color(COLS[5]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color(COLS[6]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color(COLS[7]),
                LineWidth(1.0),
            ],
        )
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fb.finish(fg);
}
*/




#[derive(Debug)]
struct RecordBench {
    bench: (f32, f32),
    bench_par: (f32, f32),
    nosort: (f32, f32),
    nosort_par: (f32, f32),
}

impl RecordBench{

    pub fn new(num_bots: usize, grow: f64, do_all: bool) -> RecordBench {
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
        }else{
            (0.0,0.0)
        };

        let nosort_par = if do_all || num_bots <= NO_SORT_MAX{
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
        }else{
            (0.0,0.0)
        };
    
        RecordBench {
            bench,
            bench_par,
            nosort,
            nosort_par,
        }
    }
    
}
