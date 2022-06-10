use support::prelude::*;

const MAX: usize = 30_000;

fn test_direct<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(usize, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    test_one_kind(&mut all)
}

fn test_indirect<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(usize, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    let mut all: Vec<_> = all.iter_mut().collect();
    test_one_kind(&mut all)
}

fn test_default<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(usize, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    let mut all: Vec<_> = all.iter_mut().map(|x| Dummy(x.0, &mut x.1)).collect();
    test_one_kind(&mut all)
}

fn test_one_kind<T: ColfindHandler>(all: &mut [T]) -> Vec<(usize, f64, f64)> {
    let mut plots = vec![];
    for a in n_iter(0, MAX) {
        let bots = &mut all[0..a];

        let (mut tree, construct_time) = bench_closure_ret(|| broccoli::Tree::new(bots));

        let (_tree, query_time) = bench_closure_ret(|| {
            tree.find_colliding_pairs(T::handle);
            tree
        });

        plots.push((a, construct_time, query_time));
    }
    plots
}

pub enum Layout {
    Direct,
    Indirect,
    Default,
}

#[inline(never)]
pub fn bench(typ: Layout, grow: f64, size: usize) -> Vec<(usize, f64, f64)> {
    match typ {
        Layout::Direct => match size {
            8 => test_direct(grow, [0u8; 8]),
            128 => test_direct(grow, [0u8; 128]),
            256 => test_direct(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
        Layout::Indirect => match size {
            8 => test_indirect(grow, [0u8; 8]),
            128 => test_indirect(grow, [0u8; 128]),
            256 => test_indirect(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
        Layout::Default => match size {
            8 => test_default(grow, [0u8; 8]),
            128 => test_default(grow, [0u8; 128]),
            256 => test_default(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
    }
}
