use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<i32>,
}

struct Res {
    num_pairs: usize,
    num_comparison: usize,
}

fn test1(grow:f64,inner:&mut [isize]) -> Res {
    
    let (num_pairs, num_comparison) = datanum::datanum_test_ret(|maker| {
            

        let mut bots = distribute(grow,inner,|a|a.to_isize_dnum(maker));

        let mut tree = TreeBuilder::new(&mut bots).build_seq();

        let mut num_pairs = 0;

        tree.new_colfind_builder().query_seq(|_a, _b| {
            num_pairs += 1;
        });
        num_pairs
    });

    Res {
        num_pairs,
        num_comparison,
    }
}

fn test2(grow:f64,inner:&mut [isize]) -> Res {
    
    let (num_pairs, num_comparison) = datanum::datanum_test_ret(|maker| {
            
        let mut bb = distribute(grow,inner,|a|a.to_isize_dnum(maker));

        let mut num_pairs = 0;
        find_collisions_sweep_mut(&mut bb, axgeom::XAXIS, |_a, _b| {
            num_pairs += 1;
        });


        num_pairs
    });

    Res {
        num_pairs,
        num_comparison,
    }
}

fn test3(grow:f64,inner:&mut [isize]) -> Res {
   

    let (num_pairs, num_comparison) = datanum::datanum_test_ret(|maker| {
            
        let mut bb = distribute(grow,inner,|a|a.to_isize_dnum(maker));
        let mut num_pairs = 0;
        NaiveAlgs::from_slice(&mut bb).find_colliding_pairs_mut(|_a, _b| {
            num_pairs += 1;
        });

        num_pairs
    });

    Res {
        num_pairs,
        num_comparison,
    }
}

fn test4(grow:f64,inner:&mut [isize]) -> Res {
    

    let (num_pairs, num_comparison) = datanum::datanum_test_ret(|maker| {
        
        let mut bots = distribute(grow,inner,|a|a.to_isize_dnum(maker));
        let mut tree = NotSorted::new_par(&mut bots);

        let mut num_pairs = 0;

        tree.find_colliding_pairs_mut(|_a, _b| {
            num_pairs += 1;
        });
        num_pairs
    });

    Res {
        num_pairs,
        num_comparison,
    }
}

//fn make<'a,'b>(grow:f64, maker: &'b datanum::Maker,bot_inner:&'a mut [isize])->Vec<BBox<datanum::Dnum<'b,isize>, &'a mut isize>>{
//}

#[derive(Debug)]
struct Record {
    num_bots: usize,
    grow: f32,
    num_pairs: usize,
    z1: usize,
    z2: usize,
    z3: usize,
    z4: usize,
}

fn handle_spiral(fb: &mut FigureBuilder) {
    let mut rects = Vec::new();

    for num_bots in (0..6000).step_by(500) {
        for grow in (0usize..50).map(|a| {
            let a: f64 = a as f64;
            0.0005 + a * 0.0002
        }) {
            
            let mut scene: Vec<_> = (0..num_bots).map(|_| 0isize).collect();
            
            let z1 = test1(grow,&mut scene);
            let z2 = test2(grow,&mut scene);
            let z3 = test3(grow,&mut scene);
            let z4 = test4(grow,&mut scene);

            //black_box(scene.bots.drain(..).map(|a| a.num).count());
            black_box(scene);

            let num_pairs = {
                assert_eq!(z1.num_pairs, z2.num_pairs);
                assert_eq!(z2.num_pairs, z3.num_pairs);
                z1.num_pairs
            };

            let z1 = z1.num_comparison;
            let z2 = z2.num_comparison;
            let z3 = z3.num_comparison;
            let z4 = z4.num_comparison;
            let r = Record {
                num_bots,
                grow:grow as f32,
                num_pairs,
                z1,
                z2,
                z3,
                z4,
            };
            rects.push(r);
        }
    }
    draw_rects(&mut rects, fb, "3d_colfind_num_pairs");
}

fn draw_rects(rects: &mut [Record], fb: &mut FigureBuilder, name1: &str) {
    {
        let x = rects.iter().map(|a| a.num_bots as f32);
        let y = rects.iter().map(|a| a.grow);
        let z1 = rects.iter().map(|a| a.z1 as f32);
        let z2 = rects.iter().map(|a| a.z2 as f32);
        let z3 = rects.iter().map(|a| a.z3 as f32);
        let z4 = rects.iter().map(|a| a.z4 as f32);

        /*
        let (x2, y2, z3) = {
            let ii = rects.iter().filter(|a| a.z3.is_some());
            let x = ii.clone().map(|a| a.num_bots as f32);
            let y = ii.clone().map(|a| a.grow as f32);
            let z3 = ii.clone().map(|a| a.z3.unwrap());

            (x, y, z3)
        };
        */

        let mut fg = fb.build(name1);

        fg.axes3d()
            .set_view(110.0, 30.0)
            .set_title("Comparison of Algs with abspiral(n,grow)", &[])
            .set_x_label("Number of Objects", &[])
            .set_y_label("Grow", &[])
            .set_z_label(
                "Number of Comparisons",
                &[Rotate(90.0), TextOffset(-3.0, 0.0)],
            )
            .points(
                x.clone(),
                y.clone(),
                z1.clone(),
                &[
                    Caption("broccoli"),
                    PointSymbol('O'),
                    Color(COLS[0]),
                    PointSize(1.0),
                ],
            )
            .points(
                x.clone(),
                y.clone(),
                z2.clone(),
                &[
                    Caption("Sweep and Prune"),
                    PointSymbol('o'),
                    Color(COLS[1]),
                    PointSize(1.0),
                ],
            )
            .points(
                x.clone(),
                y.clone(),
                z3.clone(),
                &[
                    Caption("Naive"),
                    PointSymbol('o'),
                    Color(COLS[2]),
                    PointSize(0.5),
                ],
            )
            .points(
                x.clone(),
                y.clone(),
                z4.clone(),
                &[
                    Caption("KdTree"),
                    PointSymbol('o'),
                    Color(COLS[3]),
                    PointSize(0.5),
                ],
            );

        fb.finish(fg);
    }
}

pub fn handle(fb: &mut FigureBuilder) {
    handle_spiral(fb);
}
