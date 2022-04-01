//! Contains query modules for each query algorithm.

use broccoli_tree::default_axis;
use crate::halfpin::*;
use crate::node::*;
use crate::util::*;
use alloc::vec::Vec;
use axgeom::*;
use compt::*;

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod rect;

mod tools;
