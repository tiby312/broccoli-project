use crate::inner_prelude::*;


#[derive(Debug,Serialize)]
struct Record {
    float: f32,
    float_par: f32,
    int: f32,
    int_par: f32,
    i64: f32,
    i64_par: f32,
    float_i32: f32,
}

impl Record{
    fn new(grow:f32,num_bots:usize)->Record{
        let grow=grow as f64;
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
            let bb = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let border = compute_border(&bb).unwrap();

            bench_closure(|| {
                let mut bb = convert_dist(bb, |a| convert::rect_f32_to_u32(a, &border));

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
                let mut tree = broccoli::new_par(RayonJoin, &mut bb);

                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_integer_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_f64n());

            bench_closure(|| {
                let mut tree = broccoli::new_par(RayonJoin, &mut bb);

                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };

        let bench_i64_par = {
            let mut bb = distribute(grow, &mut bot_inner, |a| a.to_i64());

            bench_closure(|| {
                let mut tree = broccoli::new_par(RayonJoin, &mut bb);

                tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                });
            })
        };


        Record {
            i64:bench_i64 as f32,
            i64_par:bench_i64_par as f32,
            float:bench_float as f32,
            int:bench_integer as f32,
            float_par:bench_float_par as f32,
            int_par:bench_integer_par as f32,
            float_i32:bench_float_i32 as f32,
        }

    }
}


pub fn handle(fb: &mut FigureBuilder) {

    fb.make_graph(Args {
        filename: "float_vs_integer",
        title: "Bench of differing number types With abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(10,10_000).map(|n| (n as f32, Record::new(0.2, n))),
        stop_values: &[],
    });
    
}