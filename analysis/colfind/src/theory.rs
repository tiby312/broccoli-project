use super::*;

#[derive(Debug)]
pub struct Record {
    pub brocc: f64,
    pub naive: f64,
    pub sweep: f64,
    pub nosort: f64,
}

pub fn new_record<T: ColfindHandler>(
    man: &mut datanum::DnumManager,
    bots: &mut [T],
    nosort: bool,
    naive: bool,
    sweep: bool,
) -> Record {
    let recorder = man;
    let c1 = recorder.time(|| {
        let mut tree = broccoli::Tree::new(bots);
        tree.find_colliding_pairs(T::handle);
    });

    let c2 = if naive {
        recorder.time(|| {
            Naive::new(bots).find_colliding_pairs(T::handle);
        })
    } else {
        0
    };

    let c3 = if sweep {
        recorder.time(|| {
            broccoli::SweepAndPrune::new(bots).find_colliding_pairs(T::handle);
        })
    } else {
        0
    };

    let c4 = if nosort {
        recorder.time(|| {
            let _tree = NotSortedTree::new(bots).find_colliding_pairs(T::handle);
        })
    } else {
        0
    };

    Record {
        brocc: c1 as f64,
        naive: c2 as f64,
        sweep: c3 as f64,
        nosort: c4 as f64,
    }
}
