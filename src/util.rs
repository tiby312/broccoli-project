use crate::pmut::PMut;
use crate::pmut::PMutPtr;
use itertools::Itertools;
use alloc::vec::Vec;

///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Default)]
pub struct PreVecMut<T> {
    vec: Vec<PMutPtr<T>>,
}

impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        PreVecMut { vec: Vec::new() }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a, 'b: 'a>(&'a mut self) -> &'a mut Vec<PMut<'b, T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe { &mut *(v as *mut _ as *mut Vec<_>) }
    }
}





///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplitMut<'a, T, F> {
    arr: Option<&'a mut [T]>,
    func: F,
}

impl<'a, T, F: FnMut(&T, &T) -> bool> SliceSplitMut<'a, T, F> {
    pub fn new(arr: &'a mut [T], func: F) -> SliceSplitMut<'a, T, F> {
        SliceSplitMut {
            arr: Some(arr),
            func,
        }
    }
}

impl<'a, T, F: FnMut(&T, &T) -> bool> DoubleEndedIterator for SliceSplitMut<'a, T, F> {
    fn next_back(&mut self)->Option<Self::Item>{
        let (last, arr) = {
            let arr = self.arr.take()?;
            let ll=arr.len();
            let i = arr.last()?;
            let count = arr.iter().rev().peeking_take_while(|a| (self.func)(a, i)).count();
            (ll-count, arr)
        };
        let (rest, last) = arr.split_at_mut(last);
        self.arr = Some(rest);
        Some(last)
        
    }
}
impl<'a, T, F: FnMut(&T, &T) -> bool> Iterator for SliceSplitMut<'a, T, F> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        let (last, arr) = {
            let arr = self.arr.take()?;
            let i = arr.get(0)?;
            let count = arr.iter().peeking_take_while(|a| (self.func)(a, i)).count();
            (count, arr)
        };
        let (first, rest) = arr.split_at_mut(last);
        self.arr = Some(rest);
        Some(first)
    }
}

/*
///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplit<'a, T, F> {
    arr: Option<&'a [T]>,
    func: F,
}
impl<'a, T, F: FnMut(&T, &T) -> bool> SliceSplit<'a, T, F> {
    pub fn new(arr: &'a [T], func: F) -> SliceSplit<'a, T, F> {
        SliceSplit {
            arr: Some(arr),
            func,
        }
    }
}
impl<'a, T, F: FnMut(&T, &T) -> bool> Iterator for SliceSplit<'a, T, F> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        let (last, arr) = {
            let arr = self.arr.take()?;
            let i = arr.get(0)?;
            let count = arr.iter().peeking_take_while(|a| (self.func)(a, i)).count();
            (count, arr)
        };
        let (first, rest) = arr.split_at(last);
        self.arr = Some(rest);
        Some(first)
    }
}
*/