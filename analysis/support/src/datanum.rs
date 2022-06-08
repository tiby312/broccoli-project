use broccoli::tree::node::Num;
use once_cell::race::OnceBool;
use std::cmp::Ordering;

use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct Dnum<I: Num>(pub I, PhantomData<*mut usize>);

impl<T: Num + Default> Default for Dnum<T> {
    fn default() -> Self {
        Dnum(Default::default(), PhantomData)
    }
}
impl<I: Num> PartialOrd for Dnum<I> {
    fn partial_cmp(&self, other: &Dnum<I>) -> Option<Ordering> {
        unsafe {
            COUNTER += 1;
        }
        self.0.partial_cmp(&other.0)
    }
}

impl<I: Num> PartialEq for Dnum<I> {
    fn eq(&self, other: &Dnum<I>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<I: Num> Eq for Dnum<I> {}

static mut COUNTER: usize = 0;

use std::fmt::Debug;

use crate::Recorder;

//You can only make this struct ONCE which
//will destine all numbers created using this struct
//to belong to the read used to call this function.
pub fn new_session() -> DnumManager {
    static INSTANCE: OnceBool = OnceBool::new();

    assert!(INSTANCE.get().is_none());

    INSTANCE.set(true).unwrap();

    DnumManager { _p: PhantomData }
}

#[derive(Debug)]
pub struct DnumManager {
    _p: PhantomData<*mut usize>,
}

impl DnumManager {
    pub fn reset_counter(&mut self) {
        unsafe { COUNTER = 0 }
    }
    pub fn counter(&self) -> usize {
        unsafe { COUNTER }
    }
    pub fn make_num<I: PartialOrd + Copy + Default + Debug>(&mut self, a: I) -> Dnum<I> {
        Dnum(a, PhantomData)
    }
}

impl Recorder<usize> for DnumManager {
    fn time_ext<K>(&mut self, func: impl FnOnce() -> K) -> (K, usize) {
        unsafe { COUNTER = 0 };
        let k = func();
        (k, unsafe { COUNTER })
    }
}
