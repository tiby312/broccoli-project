use crate::*;

use core::cmp::Ordering;

pub fn is_sorted_by<I, F>(arr: &[I], mut compare: F) -> bool
where
    F: FnMut(&I, &I) -> Option<Ordering>,
{
    arr.windows(2)
        .all(|w| compare(&w[1], &w[0]).unwrap() != Ordering::Less)
}

#[inline(always)]
pub fn combine_slice<'a, T>(a: &'a [T], b: &'a [T]) -> &'a [T] {
    let alen = a.len();
    let blen = b.len();
    unsafe {
        assert_eq!(
            a.as_ptr().add(a.len()),
            b.as_ptr(),
            "Slices are not continuous"
        );

        core::slice::from_raw_parts(a.as_ptr(), alen + blen)
    }
}

#[inline(always)]
pub fn empty_slice_from_mut<'a, 'b, T>(a: &'a mut [T]) -> &'b mut [T] {
    assert!(a.is_empty());
    unsafe { core::slice::from_raw_parts_mut(a.as_mut_ptr(), 0) }
}

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

pub use self::prevec::PreVec;

mod prevec {
    use crate::pmut::PMut;
    use alloc::vec::Vec;
    use twounordered::TwoUnorderedVecs;

    //The vec is guaranteed to be empty..
    unsafe impl<T: Send> core::marker::Send for PreVec<T> {}
    unsafe impl<T: Sync> core::marker::Sync for PreVec<T> {}

    ///An vec api to avoid excessive dynamic allocation by reusing a Vec
    pub struct PreVec<T> {
        vec: TwoUnorderedVecs<*mut T>,
    }

    impl<T> PreVec<T> {
        #[allow(dead_code)]
        #[inline(always)]
        pub fn new() -> PreVec<T> {
            PreVec {
                vec: TwoUnorderedVecs::new(),
            }
        }
        #[inline(always)]
        pub fn with_capacity(num: usize) -> PreVec<T> {
            PreVec {
                vec: TwoUnorderedVecs::with_capacity(num),
            }
        }

        ///Take advantage of the big capacity of the original vec.
        pub fn extract_two_vec<'b>(&mut self) -> TwoUnorderedVecs<PMut<'b, T>> {
            assert!(self.vec.as_vec().is_empty());
            let mut v = TwoUnorderedVecs::new();
            core::mem::swap(&mut v, &mut self.vec);
            assert!(self.vec.as_vec().is_empty());
            unsafe { v.convert() }
        }

        ///Take advantage of the big capacity of the original vec.
        pub fn extract_vec<'a, 'b>(&'a mut self) -> Vec<PMut<'b, T>> {
            assert!(self.vec.as_vec().is_empty());
            self.extract_two_vec().replace_inner(Vec::new()).0
        }

        ///Return the big capacity vec
        pub fn insert_vec(&mut self, vec: Vec<PMut<'_, T>>) {
            assert!(self.vec.as_vec().is_empty());
            let v = TwoUnorderedVecs::from_vec(vec);
            let mut v = unsafe { v.convert() };
            core::mem::swap(&mut self.vec, &mut v)
        }

        ///Return the big capacity vec
        pub fn insert_two_vec(&mut self, v: TwoUnorderedVecs<PMut<'_, T>>) {
            assert!(self.vec.as_vec().is_empty());
            let mut v = unsafe { v.convert() };
            core::mem::swap(&mut v, &mut self.vec);
        }
    }

    #[test]
    fn test_prevec() {
        use axgeom::*;
        let mut rects = vec![rect(0usize, 0, 0, 0); 100];

        let mut k = PreVec::with_capacity(0);
        for (i, r) in rects.chunks_mut(2).enumerate() {
            let (a, rest) = r.split_first_mut().unwrap();
            let (b, _) = rest.split_first_mut().unwrap();

            let mut j = k.extract_two_vec();
            if i % 2 == 0 {
                j.second().push(PMut::new(a));
                j.first().push(PMut::new(b));
            } else {
                j.first().push(PMut::new(a));
                j.second().push(PMut::new(b));
            }
            j.clear();
            k.insert_two_vec(j);
        }

        {
            let mut j = k.extract_vec();
            for a in rects.iter_mut() {
                j.push(PMut::new(a));
            }
            j.clear();
            k.insert_vec(j);
        }
        assert_eq!(k.vec.as_vec().capacity(), 128);
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
        #[inline(always)]
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
