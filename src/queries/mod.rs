//! Contains query modules for each query algorithm.

use super::*;
use crate::aabb_pin::*;
use alloc::vec::Vec;
use axgeom::*;
use compt::*;
use tree::default_axis;

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod rect;

pub mod intersect_with;

mod tools;

pub mod nbody;

impl<'a, T: Aabb> Tree2<'a, T> {
    ///
    /// If the top 25% of tree levels has more elements than the bottom 75%,
    /// consider the tree not good for querying without
    /// a considerable jump in computation cost.
    ///
    /// If worst case n*n time is unacceptable for your usecase, consider calling this
    /// after tree construction. If the tree is degenerate, consider just handling
    /// a subset of all colliding pairs by using the colliding pair building blocks
    /// in [`queries::colfind::build`]
    ///
    #[must_use]
    pub fn is_degenerate(&self) -> bool {
        let tree = self;
        //TODO test for tree with nun level 1.
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
