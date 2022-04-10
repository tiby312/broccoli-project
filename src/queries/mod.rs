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
/// Iterate over every pair regardless if colliding or not.
///
pub fn for_every_pair<T: Aabb>(
    mut arr: AabbPin<&mut [T]>,
    mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
) {
    loop {
        let temp = arr;
        match temp.split_first_mut() {
            Some((mut b1, mut x)) => {
                for mut b2 in x.borrow_mut().iter_mut() {
                    func(b1.borrow_mut(), b2.borrow_mut());
                }
                arr = x;
            }
            None => break,
        }
    }
}
