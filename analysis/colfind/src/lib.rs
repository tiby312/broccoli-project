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

#[inline(never)]
pub fn bench(max:usize,grow: f64,naive_stop:usize,sweep_stop:usize) -> Vec<(usize, Record)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max).step_by(100).map(|a|{
        let bots = &mut all[0..a];
        (a, new_record(bots, a<naive_stop, a<sweep_stop))
    }).collect()
}

#[inline(never)]
pub fn bench_grow(num: usize,start_grow:f64,end_grow:f64) -> Vec<(f64, Record)> {
    grow_iter(start_grow,end_grow).map(|grow|{
        let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
        (grow,new_record(&mut all, false, true))
    }).collect()    
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
