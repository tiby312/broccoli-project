use broccoli::axgeom;
use broccoli::axgeom::*;
use std::time::Duration;
use std::time::Instant;

#[inline(never)]
pub fn black_box_ret<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

#[inline(never)]
pub fn black_box<T>(dummy: T) {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        std::mem::forget(ret);
    }
}

pub fn bench_closure(func: impl FnOnce()) -> f64 {
    black_box_ret(bench_closure_ret(func).1)
}

pub fn instant_to_sec(elapsed: Duration) -> f64 {
    let secs: f64 = elapsed.as_secs() as f64;
    let nano: f64 = elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0
}

pub fn bench_closure_ret<T>(func: impl FnOnce() -> T) -> (T, f64) {
    let instant = Instant::now();
    let a = black_box_ret(func());
    let j = instant_to_sec(instant.elapsed());
    (a, j)
}

fn par_bench(num_bots: usize, grow: f64) {
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let s = dists::fib_iter([0.0, 0.0], grow).take(num_bots);

    let mut bots: Vec<_> = bot_inner
        .iter_mut()
        .zip(s)
        .map(|(b, a)| {
            (
                axgeom::Rect::from_point(vec2(a[0], a[1]), vec2same(RADIUS as f64)),
                b,
            )
        })
        .collect();

    for _ in 0..30 {
        let (mut tree, seq_c) = bench_closure_ret(|| broccoli::Tree::new(&mut bots));

        let seq_q = bench_closure(|| {
            tree.find_colliding_pairs(|a, b| {
                **a.unpack_inner() ^= 1;
                **b.unpack_inner() ^= 1;
            });
        });

        let (mut tree, par_c) = bench_closure_ret(|| broccoli::Tree::par_new(&mut bots));

        let par_q = bench_closure(|| {
            tree.par_find_colliding_pairs(|a, b| {
                **a.unpack_inner() ^= 1;
                **b.unpack_inner() ^= 1;
            });
        });
        let create_speedup = seq_c as f64 / par_c as f64;
        let query_speedup = seq_q as f64 / par_q as f64;

        println!(
            "create/query speedup: {:.2}x {:.2}x",
            create_speedup, query_speedup
        );
    }

    black_box(bot_inner);
}
/*
fn cmp_bench(){
    let grow = DEFAULT_GROW;
    let num_bots = 20_000;
    use crate::support::*;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    for _ in 0..2 {
        let c0 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let mut tree = broccoli::Tree::new(&mut bots);

            let mut num_collide = 0;

            tree.find_colliding_pairs(|a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
                num_collide += 1;
            });
            dbg!(num_collide);
        });

        dbg!(c0);
    }
}
*/

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args[1].as_ref() {
        "par-bench" => {
            par_bench(args[2].parse().unwrap(), args[3].parse().unwrap());
        }
        _ => println!("invalid arg"),
    }
}

pub const RADIUS: f32 = 2.0;

//abspiral(20_000,2.1)~=20_000
//abspiral(20_000,1.5)~=3*20_000
//abspiral(20_000,0.6)~=20*20_000
//abspiral(20_000,0.2)~=180*20_000
pub const DEFAULT_GROW: f64 = 1.5;
pub const DENSE_GROW: f64 = 0.6;
pub const MEGA_DENSE_GROW: f64 = 0.2;
pub const MEGA_MEGA_DENSE_GROW: f64 = 0.02;

pub const SPARSE_GROW: f64 = 2.1;
