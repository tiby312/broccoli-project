use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder){
    handle_optimal(0.2,fb);
    handle_broccoli(0.2,fb);
}

pub fn handle_broccoli(grow:f64,fb: &mut FigureBuilder){

    struct BenchRes{
        num_bots:usize,
        bench:f64,
        bench_par:f64,
        collect:f64,
        collect_par:f64,
        
    }

    let rects:Vec<_>=(0..40_000usize).step_by(50).map(|num_bots|
        {
            
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();
            
            let bench = {
                
                let mut tree =
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                bench_closure(|| {
                    tree.find_colliding_pairs_mut(|a, b| {
                        **a.unpack_inner()+=1;
                        **b.unpack_inner()+=1;
                    });
                })
            };

            let bench_par = {
                
                let mut tree =
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                    bench_closure(|| {
                    tree.find_colliding_pairs_mut_par(|a, b| {
                        **a.unpack_inner()+=1;
                        **b.unpack_inner()+=1;
                    });
                })
            };
            

            let collect = {
                
                let mut tree =
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                bench_closure(|| {
                    let c= tree.collect_colliding_pairs(|a, b| {
                        *a+=1;
                        *b+=1;
                        Some(())
                    });
                    black_box(c);
                })
            };

            let collect_par = {
                
                let mut tree =
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                    bench_closure(|| {
                        let c=tree.collect_colliding_pairs_par(|a, b| {
                            *a+=1;
                            *b+=1;
                            Some(())
                        });
                        black_box(c);
                    })
            };
            

            

            black_box(bot_inner);

            BenchRes{
                num_bots,
                bench,
                bench_par,
                collect,
                collect_par
            }
        }
    ).collect();

    
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.bench);
    let y2 = rects.iter().map(|a| a.bench_par);
    let y3 = rects.iter().map(|a| a.collect);
    let y4 = rects.iter().map(|a| a.collect_par);


    let mut fg = fb.build("broccoli_query");
    
    let linew=1.0;
    fg.axes2d()
        .set_title("broccoli query with abspiral(0.2,n)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[
                Caption("broccoli"),
                Color(COLS[0]),
                LineWidth(linew),
            ],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("broccoli par"), Color(COLS[1]), LineWidth(linew)],
        )
        .lines(
            x.clone(),
            y3,
            &[
                Caption("collect"),
                Color(COLS[2]),
                LineWidth(linew),
            ],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("collect par"), Color(COLS[3]), LineWidth(linew)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
    
    fb.finish(fg);
}

pub fn handle_optimal(grow:f64,fb: &mut FigureBuilder){
    
    struct BenchRes{
        num_bots:usize,
        optimal:f64,
        optimal_par:f64
    }

    let rects:Vec<_>=(0..40_000usize).step_by(50).map(|num_bots|
        {
            
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();
            
            let optimal={
                let mut tree= 
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                let mut pairs=tree.collect_colliding_pairs(|_,_|{
                    Some(())
                });

                bench_closure(|| {
                    pairs.for_every_pair_mut(&mut bot_inner,|a, b,()| {
                        *a+=1;
                        *b+=1;
                    });
                })
            };


            let optimal_par={
                let mut tree= 
                    crate::support::make_tree_ref_ind(&mut bot_inner,grow,|a|a.to_f32n());
                
                let mut pairs=tree.collect_colliding_pairs_par(|_,_|{
                    Some(())
                });

                bench_closure(|| {
                    pairs.for_every_pair_mut_par(&mut bot_inner,|a, b,()| {
                        *a+=1;
                        *b+=1;
                    });
                })
            };

            black_box(bot_inner);

            BenchRes{
                num_bots,
                optimal,
                optimal_par
            }
        }
    ).collect();

    
    let x = rects.iter().map(|a| a.num_bots);
    let y3 = rects.iter().map(|a| a.optimal);
    let y4 = rects.iter().map(|a| a.optimal_par);


    let mut fg = fb.build("optimal_query");
    
    let linew=1.0;
    fg.axes2d()
        .set_title("optimal query with abspiral(0.2,n)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y3,
            &[
                Caption("optimal"),
                Color(COLS[2]),
                LineWidth(linew),
            ],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("optimal par"), Color(COLS[3]), LineWidth(linew)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
    
    fb.finish(fg);
      
}