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
