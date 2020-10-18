use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;
#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<i32>,
    num: usize,
}

pub fn handle_bench_inner(scene: &mut bot::BotScene<Bot>, height: usize) -> f64 {
    
    bench_closure(||{
        let bots = &mut scene.bots;
        let prop = &scene.bot_prop;
        let mut bb = bbox_helper::create_bbox_mut(bots, |b| prop.create_bbox_i32(b.pos));
    
        let mut tree = TreeBuilder::new(&mut bb).with_height(height).build_seq();
    
        tree.find_colliding_pairs_mut(|a, b| {
            a.num += 2;
            b.num += 2;
        });
    
    })
    
}

pub fn handle_theory_inner(scene: &mut bot::BotScene<Bot>, height: usize) -> usize {
    
    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    datanum::datanum_test(|maker|{
        let mut bb = bbox_helper::create_bbox_mut(bots, |b| {
            maker.from_rect( prop.create_bbox_i32(b.pos))
        });
        let mut tree = TreeBuilder::new(&mut bb).with_height(height).build_seq();
    
        tree.find_colliding_pairs_mut(|a, b| {
            a.num += 2;
            b.num += 2;
        });  
    })
}

pub fn handle(fb: &mut FigureBuilder) {
    handle2d(fb);
    handle_lowest(fb);
}

fn handle_lowest(fb: &mut FigureBuilder) {
    struct BenchRecord {
        height: usize,
        num_bots: usize,
    }

    /*
    struct TheoryRecord {
        height: usize,
        num_bots: usize,
    }
    */

    let mut benches: Vec<BenchRecord> = Vec::new();
    //let mut theories: Vec<TheoryRecord> = Vec::new();

    let its = (1usize..80_000).step_by(2000);
    for num_bots in its.clone() {
        let mut minimum = None;
        //let mut minimum_theory = None;
        let max_height = (num_bots as f64).log2() as usize;

        let mut scene = bot::BotSceneBuilder::new(num_bots).build_specialized(|_, pos| Bot {
            pos: pos.inner_as(),
            num: 0,
        });

        for height in 1..max_height {
            //let theory = handle_theory_inner(&mut scene, height);
            let bench = handle_bench_inner(&mut scene, height);
            match minimum {
                Some((a, _b)) => {
                    if bench < a {
                        minimum = Some((bench, height));
                    }
                }
                None => {
                    minimum = Some((bench, height));
                }
            }
            /*
            match minimum_theory {
                Some((a, _b)) => {
                    if theory < a {
                        minimum_theory = Some((theory, height));
                    }
                }
                None => {
                    minimum_theory = Some((theory, height));
                }
            }
            */
        }

        /*
        if let Some((_, height)) = minimum_theory {
            theories.push(TheoryRecord { height, num_bots });
            let (_, height) = minimum.unwrap();
            benches.push(BenchRecord { height, num_bots });
        }
        */
        if let Some((_, height)) = minimum {
            benches.push(BenchRecord { height, num_bots });
        }
    }

    {
        let mut fg = fb.build("height_heuristic_vs_optimal");

        //let xx = theories.iter().map(|a| a.num_bots);
        //let yy = theories.iter().map(|a| a.height);

        let x = benches.iter().map(|a| a.num_bots);
        let y = benches.iter().map(|a| a.height);

        let heur = {
            let mut vec = Vec::new();
            for num_bots in its.clone() {
                let height = compute_tree_height_heuristic(num_bots, DEFAULT_NUMBER_ELEM_PER_NODE);
                vec.push((num_bots, height));
            }
            vec
        };

        let heurx = heur.iter().map(|a| a.0);
        let heury = heur.iter().map(|a| a.1);

        /*
        fg.axes2d()
            .set_pos_grid(2, 1, 0)
            .set_legend(
                Graph(1.0),
                Graph(0.0),
                &[LegendOption::Placement(AlignRight, AlignBottom)],
                &[],
            )
            .set_title(
                "Dinotree Colfind Query: Optimal Height vs Heuristic Height with abspiral(x,2.0)",
                &[],
            )
            .set_x_label("Num bots", &[])
            .set_y_label("Best Tree Height", &[])
            .points(
                xx,
                yy,
                &[
                    Caption("Optimal"),
                    PointSymbol('O'),
                    Color("red"),
                    PointSize(1.0),
                ],
            )
            .points(
                heurx.clone(),
                heury.clone(),
                &[
                    Caption("Heuristic"),
                    PointSymbol('x'),
                    Color("blue"),
                    PointSize(2.0),
                ],
            );
        */
        fg.axes2d()
            //.set_pos_grid(2, 1, 1)
            .set_legend(
                Graph(1.0),
                Graph(0.0),
                &[LegendOption::Placement(AlignRight, AlignBottom)],
                &[],
            )
            .set_title(
                "Dinotree Colfind Query: Optimal Height vs Heuristic Height with abspiral(x,2.0)",
                &[],
            )
            .set_x_label("Num bots", &[])
            .set_y_label("Best Tree Height", &[])
            .points(
                x,
                y,
                &[
                    Caption("Optimal"),
                    PointSymbol('O'),
                    Color("red"),
                    PointSize(1.0),
                ],
            )
            .points(
                heurx,
                heury,
                &[
                    Caption("Heuristic"),
                    PointSymbol('x'),
                    Color("blue"),
                    PointSize(2.0),
                ],
            );

        fb.finish(fg);
    }
}

fn handle2d(fb: &mut FigureBuilder) {
    #[derive(Debug)]
    struct Record {
        height: usize,
        num_comparison: usize,
    }

    #[derive(Debug)]
    struct BenchRecord {
        height: usize,
        bench: f64,
    }

    let mut theory_records = Vec::new();

    let mut bench_records: Vec<BenchRecord> = Vec::new();

    let mut scene = bot::BotSceneBuilder::new(10_000).build_specialized(|_, pos| Bot {
        pos: pos.inner_as(),
        num: 0,
    });

    for height in 2..13 {
        let num_comparison = handle_theory_inner(&mut scene, height);
        theory_records.push(Record {
            height,
            num_comparison,
        });
    }

    for height in (2..13).flat_map(|a| std::iter::repeat(a).take(20)) {
        let bench = handle_bench_inner(&mut scene, height);
        bench_records.push(BenchRecord { height, bench });
    }

    use gnuplot::*;

    let mut fg = fb.build("height_heuristic");

    let x = theory_records.iter().map(|a| a.height);
    let y = theory_records.iter().map(|a| a.num_comparison);

    fg.axes2d()
        .set_pos_grid(2, 1, 0)
        .set_title("Number of Comparisons with different numbers of objects per node with abspiral(10000,2.0)", &[])
        .lines(x,y,&[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Number of Comparisons", &[]);

    let x = bench_records.iter().map(|a| a.height);
    let y = bench_records.iter().map(|a| a.bench);

    fg.axes2d()
        .set_pos_grid(2, 1, 1)
        .set_title("Bench times with different numbers of objects per node (seq,colfind) with abspiral(10000,2.0)", &[])
        .points(x,y,&[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Time in seconds", &[]);

    fb.finish(fg);
}
