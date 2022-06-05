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

use broccoli::tree::node::Aabb;
use broccoli::tree::node::ManySwappable;
use broccoli::tree::node::ManySwap;

fn foo(num_bots:usize,grow:f64)->Vec<ManySwappable<Rect<f32>>>{
    let s = dists::fib_iter([0.0, 0.0], grow).take(num_bots);

    let mut bots: Vec<_> = s
        .map(|a| {
            ManySwappable(
                axgeom::Rect::from_point(vec2(a[0] as f32, a[1] as f32), vec2same(RADIUS as f32)),
                )
        })
        .collect();

    bots
}

fn par_bench<T:Aabb+ManySwap+Send>(bots:&mut [T],c_num_seq_fallback:usize)->f64 where T::Num:Send{
    

    let mut total_create_speedup=0.0;
    
    let num_iter=1;
    for _ in 0..num_iter {

        let seq_c={
            let ( tree, seq_c) = bench_closure_ret(|| broccoli::Tree::new(bots));
            black_box(tree);
            seq_c
        };

        let mut build_args=broccoli::tree::BuildArgs::new(bots.len());
        build_args.num_seq_fallback=c_num_seq_fallback;

        let ( tree, par_c) = bench_closure_ret(|| broccoli::Tree::par_from_build_args(bots,build_args).0);

        let create_speedup = seq_c as f64 / par_c as f64;
        total_create_speedup+=create_speedup;

        black_box(tree);
    }
    let avg=total_create_speedup/num_iter as f64 ;
    
    avg
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
            let num_bots=args[2].parse().unwrap();
            let grow=args[3].parse().unwrap();
            let c_num_seq_fallback=args[4].parse().unwrap();
            let mut all=foo(num_bots,grow);
            let avg=par_bench(&mut all,c_num_seq_fallback);
            println!(
                "average create speedup: {:.2}x",
                avg   );
        
        }
        "par-bench-graph"=>{
            let grow=args[2].parse().unwrap();
            let c_num_seq_fallback=args[3].parse().unwrap();
            
            let mn=1000_000;
            let mut all=foo(mn,grow);
            let mut plots=vec!();
            for i in (0..mnusize).step_by(1000).skip(1){
                let ll=&mut all[0..i];
                plots.push((i as i128,par_bench(ll,c_num_seq_fallback)));
            }
            //dbg!(&plots);

            {
                use poloto::prelude::*;
                use poloto::build::scatter;
                let l1 = scatter("", &plots);
                let m = poloto::build::origin();
                let data = plots!(l1, m);

                let p = simple_fmt!(data, "hay", "x", "y");

                print!("{}", poloto::disp(|w| p.simple_theme(w)));
            }



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
