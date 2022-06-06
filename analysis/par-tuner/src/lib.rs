use support::prelude::*;




#[inline(never)]
pub fn bench_par(grow:f64,c_num_seq_fallback:Option<usize>,q_num_seq_fallback:Option<usize>)->Vec<(usize,f64,f64)>{
    
    let mn = 1_000_000;
    let mut all = dist::create_dist_manyswap(mn, grow, 0u32);
    let mut plots = vec![];
    
    for i in (0..mn).step_by(1000).skip(1) {
        let bots = &mut all[0..i];
        
        let (tree, cseq) = bench_closure_ret(|| broccoli::Tree::new(bots));
        black_box(tree);
    
        let mut args = broccoli::tree::BuildArgs::new(bots.len());
        if let Some(c)=c_num_seq_fallback{
            args.num_seq_fallback = c;
        }
        let (mut tree, cpar) = bench_closure_ret(|| broccoli::Tree::par_from_build_args(bots, args).0);
    
        let cspeedup=cseq as f64 / cpar as f64;

        let qseq = bench_closure(|| {
            tree.find_colliding_pairs(|a, b| {
                *a.unpack_inner()^=1;
                *b.unpack_inner()^=1;

            });
        });

        let mut args = broccoli::queries::colfind::QueryArgs::new();
        if let Some(c)=q_num_seq_fallback{
            args.num_seq_fallback = c;
        }
        let qpar = bench_closure(|| {
            tree.par_find_colliding_pairs_from_args(args, |a, b| {
                *a.unpack_inner()^=1;
                *b.unpack_inner()^=1;
            });
        });

        let qspeedup=qseq as f64 / qpar as f64;

        plots.push((i,cspeedup,qspeedup));
    }
    plots
}




