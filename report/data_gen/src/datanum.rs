use broccoli::axgeom::Rect;
use broccoli::Num;
use std::cmp::Ordering;

use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct Dnum<'a, I: Num>(pub I, PhantomData<&'a usize>);

impl<'a, I: Num> PartialOrd for Dnum<'a, I> {
    fn partial_cmp(&self, other: &Dnum<I>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, I: Num> PartialEq for Dnum<'a, I> {
    fn eq(&self, other: &Dnum<I>) -> bool {
        self.0.cmp(&other.0) == Ordering::Equal
    }
}

impl<'a, I: Num> Eq for Dnum<'a, I> {}
impl<'a, I: Num> Ord for Dnum<'a, I> {
    fn cmp(&self, other: &Dnum<I>) -> Ordering {
        unsafe {
            COUNTER += 1;
        }
        self.0.cmp(&other.0)
    }
}

pub static mut COUNTER: usize = 0;

///within the closure, the user is allowed to create DataNum numbers
///NOT SAFE
///The number type Dnum is incorrectly marked as Send and Sync.
///Ths is because broccoli::Num requires Send and Sync.
///It i up to the user to not move any Dnum's between threads
///inside of this closure.
pub fn datanum_test(func: impl FnOnce(&mut Maker)) -> usize {
    unsafe { COUNTER = 0 };
    let mut maker = Maker { _p: PhantomData };
    func(&mut maker);

    unsafe { COUNTER }
}

pub fn datanum_test2<T>(func: impl FnOnce(&mut Maker) -> T) -> T {
    unsafe { COUNTER = 0 };
    let mut maker = Maker { _p: PhantomData };
    func(&mut maker)
}

pub fn datanum_test_ret<T>(func: impl FnOnce(&mut Maker) -> T) -> (T, usize) {
    unsafe { COUNTER = 0 };
    let mut maker = Maker { _p: PhantomData };
    let k = func(&mut maker);
    (k, unsafe { COUNTER })
}

pub struct Maker {
    _p: PhantomData<*mut usize>, //Make it not implement send or sync
}
impl Maker {
    pub fn count(&self) -> usize {
        unsafe { COUNTER }
    }
    pub fn reset(&self){
        unsafe{COUNTER=0}
    }
    pub fn from_rect<I: Num>(&self, rect: Rect<I>) -> Rect<Dnum<I>> {
        let ((a, b), (c, d)) = rect.get();
        Rect::new(
            Dnum(a, PhantomData),
            Dnum(b, PhantomData),
            Dnum(c, PhantomData),
            Dnum(d, PhantomData),
        )
    }
}

