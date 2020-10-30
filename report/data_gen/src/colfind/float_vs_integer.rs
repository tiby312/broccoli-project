use crate::inner_prelude::*;



fn handle_bench(fg: &mut Figure) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_float: f64,
        bench_float_par: f64,
        bench_integer: f64,
        bench_integer_par: f64,
        bench_f64: f64,
        bench_f64_par: f64,
        bench_i64: f64,
        bench_i64_par: f64,
        bench_float_i32: f64,
        bench_float_ordered:f64,
        bench_float_u16_par:f64
    }

    let mut records = Vec::new();

    for num_bots in (2..80_000).step_by(200) {
        
        let grow=1.0;

        let mut bot_inner:Vec<_>=(0..num_bots).map(|_|0isize).collect();
        
        let bench_integer = {

            let mut bb:Vec<  BBox<i32,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_as()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });

            })
            
        };

        let bench_i64 = {
            
            let mut bb:Vec<  BBox<i64,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_as()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{

                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
            })
        };
        

        let bench_float_i32 = {
            
            let bb:Vec<  BBox<NotNan<f32>,&mut isize>  > =
            abspiral_f32_nan(grow).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            let border={
                let (first,rest)=bb.split_first().unwrap();
                let mut r=first.rect;
                for a in rest.iter(){
                    r.grow_to_fit(&a.rect);
                }
                r
            };



            bench_closure(||{

                let mut bb:Vec<_>=bb.into_iter().map(|a|{
                    bbox(broccoli::convert::rect_f32_to_u32(a.rect.inner_into(),&border.as_ref()),a.inner)
                }).collect();
    
                let mut tree = broccoli::new(&mut bb);
    
                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
            })
        };

        let bench_float_ordered = {
            use axgeom::ordered_float::OrderedFloat;
                
            let mut bb:Vec<  BBox<OrderedFloat<f32>,&mut isize>  > =
            abspiral_f32(grow).map(|a|a.inner_into()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
        };
        let bench_float = {
            let mut bb:Vec<  BBox<NotNan<f32>,&mut isize>  > =
            abspiral_f32_nan(grow).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
        };

        let bench_float_par = {
            let mut bb:Vec<  BBox<NotNan<f32>,&mut isize>  > =
            abspiral_f32_nan(grow).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{

                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
        };

        let bench_integer_par = {
            
            let mut bb:Vec<  BBox<i32,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_as()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();


            bench_closure(||{
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
            })
        };


        let bench_i64_par = {
            
            let mut bb:Vec<  BBox<i64,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_as()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            bench_closure(||{
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
            
        };

        let bench_f64 = {
            
            let mut bb:Vec<  BBox<NotNan<f64>,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_try_into().unwrap()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    

            bench_closure(||{
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
        };

        let bench_f64_par = {
            
            let mut bb:Vec<  BBox<NotNan<f64>,&mut isize>  > =
            abspiral_f64(grow).map(|a|a.inner_try_into().unwrap()).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    

            bench_closure(||{
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
            
        };


        let bench_float_u16_par = {
            
            let bb:Vec<  BBox<NotNan<f32>,&mut isize>  > =
            abspiral_f32_nan(grow).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();
    
            let border={
                let (first,rest)=bb.split_first().unwrap();
                let mut r=first.rect;
                for a in rest.iter(){
                    r.grow_to_fit(&a.rect);
                }
                r
            };



            bench_closure(||{

                let mut bb:Vec<_>=bb.into_iter().map(|a|{
                    bbox(broccoli::convert::rect_f32_to_u16(a.rect.inner_into(),&border.as_ref()),a.inner)
                }).collect();
    
                let mut tree = broccoli::new_par(&mut bb);
    
                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
            })
        };

        records.push(Record {
            num_bots,
            bench_i64,
            bench_i64_par,
            bench_float,
            bench_integer,
            bench_float_par,
            bench_integer_par,
            bench_f64,
            bench_f64_par,
            bench_float_i32,
            bench_float_ordered,
            bench_float_u16_par
        });
    }

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.bench_float);
    let y2 = rects.iter().map(|a| a.bench_integer);
    let y3 = rects.iter().map(|a| a.bench_float_par);
    let y4 = rects.iter().map(|a| a.bench_integer_par);
    let y5 = rects.iter().map(|a| a.bench_f64);
    let y6 = rects.iter().map(|a| a.bench_f64_par);
    let y7 = rects.iter().map(|a| a.bench_i64);
    let y8 = rects.iter().map(|a| a.bench_i64_par);
    let y9 = rects.iter().map(|a| a.bench_float_i32);
    let y10 = rects.iter().map(|a| a.bench_float_ordered);
    let y11 = rects.iter().map(|a| a.bench_float_u16_par);


    
    let ww=1.0;
    fg.axes2d()
        .set_title(
            "Comparison of broccoli Performance With Different Number Types With abspiral(x,1.0)",
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("f32"), Color("blue"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("i32"), Color("green"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("f32 parallel"), Color("red"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("i32 parallel"), Color("orange"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("f64"), Color("violet"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y6,
            &[Caption("f64 parallel"), Color("yellow"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y7,
            &[Caption("i64"), Color("brown"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y8,
            &[Caption("i64 parallel"), Color("purple"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y9,
            &[Caption("f32 to u32"), Color("black"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y10,
            &[Caption("f32 ordered"), Color("black"), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y11,
            &[Caption("f32 to u16 par"), Color("cyan"), LineWidth(ww)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
}

pub fn handle(fb: &mut FigureBuilder) {
    let mut fg = fb.build("float_vs_integer");
    handle_bench(&mut fg);
    fb.finish(fg);
}
