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
