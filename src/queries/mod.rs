//! Contains query modules for each query algorithm.

use super::*;
use crate::aabb_pin::*;
use alloc::vec::Vec;
use axgeom::*;
use broccoli_tree::default_axis;
use compt::*;

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod rect;

pub mod intersect_with;

mod tools;

pub mod nbody;

///
/// If the top 25% of tree levels has more elements than the bottom 75%,
/// consider the tree not good for querying colliding pairs without
/// a considerable jump in computation cost.
///
pub fn is_degenerate<T: Aabb>(tree: &TreeInner<Node<T>, DefaultSorter>) -> bool {
    let mut v: Vec<_> = (0..tree.vistr().get_height()).map(|_| 0).collect();

    for (i, n) in tree.vistr().with_depth(compt::Depth(0)).dfs_preorder_iter() {
        v[i.0] += n.range.len();
    }

    let total: usize = v.iter().sum();

    let relative: Vec<_> = v.iter().map(|&x| x as f64 / total as f64).collect();

    let top_20 = v.len() / 4;

    let top_20_sum: f64 = relative.iter().take(top_20).sum();
    let rest_sum: f64 = relative.iter().skip(top_20).sum();

    //dbg!(relative,top_20_sum,rest_sum);

    top_20_sum > rest_sum
}

///
/// Iterate over every pair regardless if colliding or not.
///
pub fn for_every_pair<T>(
    mut arr: AabbPin<&mut [T]>,
    mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
) {
    loop {
        let temp = arr;
        match temp.split_first_mut() {
            Some((mut b1, mut x)) => {
                x.borrow_mut()
                    .iter_mut()
                    .for_each(|mut b2| func(b1.borrow_mut(), b2.borrow_mut()));
                arr = x;
            }
            None => break,
        }
    }
}
