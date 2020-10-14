use broccoli::axgeom::Rect;
use broccoli::Num;
use std::cmp::Ordering;

pub struct Counter(usize);

pub fn from_rect<I: Num>(counter: &mut Counter, rect: Rect<I>) -> Rect<DataNum<I>> {
    let ((a, b), (c, d)) = rect.get();
    Rect::new(
        counter.new_num(a),
        counter.new_num(b),
        counter.new_num(c),
        counter.new_num(d),
    )
}

impl Counter {
    pub fn new() -> Counter {
        Counter(0)
    }
    pub fn into_inner(self) -> usize {
        self.0
    }
    pub fn get_inner(&self) -> &usize {
        &self.0
    }
    pub fn reset(&mut self) {
        self.0 = 0;
    }
    pub fn new_num<I: Num>(&mut self, a: I) -> DataNum<I> {
        DataNum(a, self as *mut Counter)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DataNum<I: Num>(pub I, *mut Counter);

//unsafe implement send and sync.
//we will be cause to only use sequential version of the tree algorithms
unsafe impl<I: Num> Send for DataNum<I> {}
unsafe impl<I: Num> Sync for DataNum<I> {}

impl<I: Num> PartialOrd for DataNum<I> {
    fn partial_cmp(&self, other: &DataNum<I>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Num> PartialEq for DataNum<I> {
    fn eq(&self, other: &DataNum<I>) -> bool {
        self.0.cmp(&other.0) == Ordering::Equal
    }
}

impl<I: Num> Eq for DataNum<I> {}
impl<I: Num> Ord for DataNum<I> {
    fn cmp(&self, other: &DataNum<I>) -> Ordering {
        unsafe {
            let p = self.1;
            (*p).0 += 1;
        }
        self.0.cmp(&other.0)
    }
}
