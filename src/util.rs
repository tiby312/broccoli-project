use itertools::Itertools;


use alloc::vec::Vec;
use crate::pmut::PMut;
//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}





///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Default)]
pub struct PreVecMut<T> {
    vec: Vec<core::ptr::NonNull<T>>,
}

impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        debug_assert_eq!(
            core::mem::size_of::<core::ptr::NonNull<T>>(),
            core::mem::size_of::<&mut T>()
        );

        PreVecMut { vec: Vec::with_capacity(1024) }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a, 'b: 'a>(&'a mut self) -> &'a mut Vec<PMut<'b, T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe { &mut *(v as *mut _ as *mut Vec<_>) }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn capacity(&self)->usize{
        self.vec.capacity()
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
    fn next_back(&mut self) -> Option<Self::Item> {
        let (last, arr) = {
            let arr = self.arr.take()?;
            let ll = arr.len();
            let i = arr.last()?;
            let count = arr
                .iter()
                .rev()
                .peeking_take_while(|a| (self.func)(a, i))
                .count();
            (ll - count, arr)
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
