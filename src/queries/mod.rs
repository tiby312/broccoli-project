//! Contains query modules for each query algorithm.

use crate::halfpin::*;
use crate::node::*;
use alloc::vec::Vec;
use axgeom::*;
use broccoli_tree::default_axis;
use compt::*;
use super::*;

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod rect;

pub mod intersect_with;

mod tools;

pub mod nbody;