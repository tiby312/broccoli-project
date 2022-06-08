use super::*;

#[derive(Debug)]
pub struct Record {
    pub broccoli: f64,
    pub naive: f64,
    pub sweep: f64,
    pub nosort: f64,
}

pub fn new_record<B: Recorder<usize>, T: ColfindHandler>(
    recorder: B,
    bots: &mut [T],
    naive: bool,
    sweep: bool,
) -> Record {
    unimplemented!();
    /*
    let recorder=Theory;
    let c1 = recorder.time(|| {
        let mut tree = broccoli::Tree::new(bots);
        tree.find_colliding_pairs(|a, b| {
            T::handle(a, b);
        });
    });

    let c2 = if naive {
        recorder.time(|| {
            Naive::new(bots).find_colliding_pairs(|a, b| {
                T::handle(a, b);
            });
        })
    } else {
        0
    };

    let c3 = if sweep {
        recorder.time(|| {
            broccoli::SweepAndPrune::new(bots).find_colliding_pairs(|a, b| {
                T::handle(a, b);
            });
        })
    } else {
        0
    };

    let c4 = recorder.time(|| {
        let _tree = NotSortedTree::new(bots).find_colliding_pairs(|a, b| {
            T::handle(a, b);
        });
    });

    Record {
        broccoli: c1 as f64,
        naive: c2 as f64,
        sweep: c3 as f64,
        nosort: c4 as f64,
    }
    */
}
