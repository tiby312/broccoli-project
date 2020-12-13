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
        bench_float_ordered: f64,
        bench_float_u16_par: f64,
    }

    let mut records = Vec::new();

    for num_bots in (50_000..120_000).step_by(200) {
        let grow = 1.0;

        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let bench_integer = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_i32());

            bench_closure(|| {
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_i64 = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_i64());

            bench_closure(|| {
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_float_i32 = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let border = compute_border(&bb).unwrap();

            bench_closure(|| {
                let mut bb = convert_dist(bb, |a| {
                    broccoli::convert::rect_f32_to_u32(a.inner_into(), &border.as_ref())
                });

                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_float_ordered = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f32ord());

            bench_closure(|| {
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };
        let bench_float = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_float_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            bench_closure(|| {
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_integer_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f64n());

            bench_closure(|| {
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_i64_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_i64());

            bench_closure(|| {
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_f64 = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f64n());

            bench_closure(|| {
                let mut tree = broccoli::new(&mut bb);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_f64_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f64n());

            bench_closure(|| {
                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_float_u16_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let border = compute_border(&bb).unwrap();

            bench_closure(|| {
                let mut bb = convert_dist(bb, |a| {
                    broccoli::convert::rect_f32_to_u16(a.inner_into(), &border.as_ref())
                });

                let mut tree = broccoli::new_par(&mut bb);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
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
            bench_float_u16_par,
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

    let ww = 1.0;
    fg.axes2d()
        .set_title(
            "Comparison of broccoli Performance With Different Number Types With abspiral(x,1.0)",
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("f32"), Color(COLS[0]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("i32"), Color(COLS[1]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("f32 parallel"), Color(COLS[2]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("i32 parallel"), Color(COLS[3]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("f64"), Color(COLS[4]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y6,
            &[Caption("f64 parallel"), Color(COLS[5]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y7,
            &[Caption("i64"), Color(COLS[6]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y8,
            &[Caption("i64 parallel"), Color(COLS[7]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y9,
            &[Caption("f32 to u32"), Color(COLS[8]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y10,
            &[Caption("f32 ordered"), Color(COLS[9]), LineWidth(ww)],
        )
        .lines(
            x.clone(),
            y11,
            &[Caption("f32 to u16 par"), Color(COLS[10]), LineWidth(ww)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
}

pub fn handle(fb: &mut FigureBuilder) {
    let mut fg = fb.build("float_vs_integer");
    handle_bench(&mut fg);
    fb.finish(fg);
}
