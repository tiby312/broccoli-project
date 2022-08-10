use super::*;

#[derive(Debug)]
pub struct Record {
    pub brocc: f64,
    pub brocc_par: f64,
    pub sweep: f64,
    //pub sweep_par: f64,
    pub naive: f64,
    pub nosort_par: f64,
    pub nosort: f64,
}

pub fn new_record<T: ColfindHandler>(
    bots: &mut [T],
    nosort_bench: bool,
    naive_bench: bool,
    sweep_bench: bool,
) -> Record
where
    T: Send,
    T::Num: Send,
{
    unimplemented!();

    // let mut recorder = Bencher;
    // let c0 = recorder.time(|| {
    //     let mut tree = broccoli::Tree::par_new(bots);

    //     tree.par_find_colliding_pairs(T::handle);
    // });

    // let c1 = recorder.time(|| {
    //     let mut tree = broccoli::Tree::new(bots);
    //     tree.find_colliding_pairs(T::handle);
    // });

    // // let c3 = if sweep_bench {
    // //     recorder.time(|| {
    // //         SweepAndPrune::new(bots).par_find_colliding_pairs(T::handle);
    // //     })
    // // } else {
    // //     0.0
    // // };

    // let c4 = if naive_bench {
    //     recorder.time(|| {
    //         Naive::new(bots).find_colliding_pairs(T::handle);
    //     })
    // } else {
    //     0.0
    // };

    // let c5 = if nosort_bench {
    //     recorder.time(|| {
    //         let mut tree = NotSortedTree::par_new(bots);

    //         tree.par_find_colliding_pairs(T::handle);
    //     })
    // } else {
    //     0.0
    // };

    // let c6 = if nosort_bench {
    //     recorder.time(|| {
    //         let mut tree = NotSortedTree::new(bots);
    //         tree.find_colliding_pairs(T::handle);
    //     })
    // } else {
    //     0.0
    // };

    // let c7 = if sweep_bench {
    //     recorder.time(|| {
    //         let mut s = broccoli::SweepAndPrune::new(bots);

    //         s.find_colliding_pairs(T::handle);
    //     })
    // } else {
    //     0.0
    // };

    // Record {
    //     brocc: c1,
    //     brocc_par: c0,
    //     //sweep_par: c3,
    //     naive: c4,
    //     nosort_par: c5,
    //     nosort: c6,
    //     sweep: c7,
    // }
}
