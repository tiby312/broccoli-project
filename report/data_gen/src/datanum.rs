use broccoli::axgeom::Rect;
use broccoli::Num;
use std::cmp::Ordering;

use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct Dnum<'a,I: Num>(pub I,PhantomData<&'a usize>);

//unsafe implement send and sync.
//we will be cause to only use sequential version of the tree algorithms
unsafe impl<'a,I: Num> Send for Dnum<'a,I> {}
unsafe impl<'a,I: Num> Sync for Dnum<'a,I> {}

impl<'a,I: Num> PartialOrd for Dnum<'a,I> {
    fn partial_cmp(&self, other: &Dnum<I>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a,I: Num> PartialEq for Dnum<'a,I> {
    fn eq(&self, other: &Dnum<I>) -> bool {
        self.0.cmp(&other.0) == Ordering::Equal
    }
}

impl<'a,I: Num> Eq for Dnum<'a,I> {}
impl<'a,I: Num> Ord for Dnum<'a,I> {
    fn cmp(&self, other: &Dnum<I>) -> Ordering {
        unsafe {
            COUNTER+=1;
        }
        self.0.cmp(&other.0)
    }
}

static mut COUNTER:usize=0;


///within the closure, the user is allowed to create DataNum numbers
///NOT SAFE
///The number type Dnum is incorrectly marked as Send and Sync.
///Ths is because broccoli::Num requires Send and Sync.
///It i up to the user to not move any Dnum's between threads 
///inside of this closure.
pub fn datanum_test(func:impl FnOnce(&mut Maker))->usize{
    unsafe{COUNTER=0};
    let mut maker=Maker{_p:PhantomData};
    func(&mut maker);

    unsafe{COUNTER}
}

pub struct Maker{
    _p:PhantomData<*mut usize> //Make it not implement send or sync
}
impl Maker{
    pub fn from_rect<I: Num>(&self,rect: Rect<I>) -> Rect<Dnum<I>> {
        let ((a, b), (c, d)) = rect.get();
        Rect::new(
            Dnum(a,PhantomData),
            Dnum(b,PhantomData),
            Dnum(c,PhantomData),
            Dnum(d,PhantomData),
        )
    }
    
}

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
