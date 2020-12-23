use crate::inner_prelude::*;

#[inline(always)]
pub fn compare_bots<T: Aabb>(axis: impl Axis, a: &T, b: &T) -> core::cmp::Ordering {
    let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
    if p1 > p2 {
        core::cmp::Ordering::Greater
    } else {
        core::cmp::Ordering::Less
    }
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering { compare_bots(axis, a, b) };

    collision_botids.sort_unstable_by(sclosure);
}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty {
    #[inline(always)]
    fn div(&mut self) -> (Self, Self) {
        (SplitterEmpty, SplitterEmpty)
    }
    #[inline(always)]
    fn add(&mut self, _: Self, _: Self) {}
}

pub use self::prevec::PreVecMut;
mod prevec {
    use crate::pmut::PMut;
    use alloc::vec::Vec;
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

            PreVecMut {
                vec: Vec::with_capacity(1024),
            }
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
        pub fn capacity(&self) -> usize {
            self.vec.capacity()
        }
    }
}

pub use self::slicesplit::SliceSplitMut;
mod slicesplit {
    use itertools::Itertools;

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
}
