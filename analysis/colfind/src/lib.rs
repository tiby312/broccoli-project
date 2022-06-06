use support::prelude::*;

#[derive(Debug)]
pub struct Record {
    pub brocc: f64,
    pub brocc_par: f64,
    pub sweep: f64,
    pub sweep_par: f64,
    pub naive: f64,
    pub nosort_par: f64,
    pub nosort: f64,
}

const MAX: usize = 30_000;
pub fn bench(grow: f64) -> Vec<(usize, Record)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(MAX).collect();

    let mut res = vec![];
    for a in (0..MAX).step_by(1000) {
        let bots = &mut all[0..a];

        res.push((a, new_record(bots, true, true)));
    }
    res
}

pub fn bench_grow(num: usize) -> Vec<(f64, Record)> {
    unimplemented!();
}

fn new_record<T: ColfindHandler>(bots: &mut [T], naive_bench: bool, sweep_bench: bool) -> Record
where
    T: Send,
    T::Num: Send,
{
    let c0 = bench_closure(|| {
        let mut tree = broccoli::Tree::par_new(bots);

        tree.par_find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
    });

    let c1 = bench_closure(|| {
        let mut tree = broccoli::Tree::new(bots);
        tree.find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
    });

    let c3 = if sweep_bench {
        bench_closure(|| {
            SweepAndPrune::new(bots)
                .par_find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
        })
    } else {
        0.0
    };

    let c4 = if naive_bench {
        bench_closure(|| {
            Naive::new(bots)
                .find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
        })
    } else {
        0.0
    };

    let c5 = bench_closure(|| {
        let mut tree = NotSortedTree::par_new(bots);

        tree.par_find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
    });

    let c6 = bench_closure(|| {
        let mut tree = NotSortedTree::new(bots);
        tree.find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
    });

    let c7 = if sweep_bench {
        bench_closure(|| {
            let mut s = broccoli::SweepAndPrune::new(bots);

            s.find_colliding_pairs(|a, b| T::handle(a.unpack_inner(), b.unpack_inner()));
        })
    } else {
        0.0
    };

    Record {
        brocc: c1,
        brocc_par: c0,
        sweep_par: c3,
        naive: c4,
        nosort_par: c5,
        nosort: c6,
        sweep: c7,
    }
}
