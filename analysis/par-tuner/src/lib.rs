use support::prelude::*;

fn single<T: ColfindHandler>(
    bots: &mut [T],
    c_num_seq_fallback: Option<usize>,
    q_num_seq_fallback: Option<usize>,
) -> (f64, f64)
where
    T: Send,
    T::Num: Send,
{
    let (tree, cseq) = bench_closure_ret(|| broccoli::Tree::new(bots));
    black_box(tree);

    let mut args = broccoli::tree::BuildArgs::new(bots.len());
    if let Some(c) = c_num_seq_fallback {
        args.num_seq_fallback = c;
    }
    let (mut tree, cpar) = bench_closure_ret(|| broccoli::Tree::par_from_build_args(bots, args).0);

    let cspeedup = cseq as f64 / cpar as f64;

    let qseq = bench_closure(|| {
        tree.find_colliding_pairs(T::handle);
    });

    let mut args = broccoli::queries::colfind::QueryArgs::new();
    if let Some(c) = q_num_seq_fallback {
        args.num_seq_fallback = c;
    }
    let qpar = bench_closure(|| {
        tree.par_find_colliding_pairs_from_args(args, T::handle);
    });

    let qspeedup = qseq as f64 / qpar as f64;

    (cspeedup, qspeedup)
}

#[inline(never)]
pub fn bench_par(
    grow: f64,
    c_num_seq_fallback: Option<usize>,
    q_num_seq_fallback: Option<usize>,
) -> Vec<(usize, f64, f64)> {
    let mn = 1_000_000;

    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(mn).collect();

    let mut plots = vec![];

    for i in (0..mn).step_by(1000).skip(1) {
        let bots = &mut all[0..i];

        let (j, k) = single(bots, c_num_seq_fallback, q_num_seq_fallback);

        plots.push((i, j, k));
    }
    plots
}


pub fn best_seq_fallback_rebal(num:usize,grow:f64)-> Vec<(usize,f64)> {
    
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (000..20_000).step_by(10).map(|r|{
        let (a,_)=single(&mut all,Some(r),None);
        (r, a as f64)
    }).collect()

}

pub fn best_seq_fallback_query(num:usize,grow:f64)->Vec<(usize,f64)> {
    
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (000..20_000).step_by(10).map(|a|{
        let (_,b)=single(&mut all,None,Some(a));
        (a, b as f64)
    }).collect()

}