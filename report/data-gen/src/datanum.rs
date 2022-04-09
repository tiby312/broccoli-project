use broccoli::axgeom::Rect;
use broccoli::tree::build::Num;
use std::cmp::Ordering;

use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct Dnum<'a, I: Num>(pub I, PhantomData<&'a usize>);

impl<'a, T: Num + Default> Default for Dnum<'a, T> {
    fn default() -> Self {
        Dnum(Default::default(), PhantomData)
    }
}
impl<'a, I: Num> PartialOrd for Dnum<'a, I> {
    fn partial_cmp(&self, other: &Dnum<I>) -> Option<Ordering> {
        unsafe {
            COUNTER += 1;
        }
        self.0.partial_cmp(&other.0)
    }
}

impl<'a, I: Num> PartialEq for Dnum<'a, I> {
    fn eq(&self, other: &Dnum<I>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, I: Num> Eq for Dnum<'a, I> {}

pub static mut COUNTER: usize = 0;

///within the closure, the user is allowed to create DataNum numbers
///NOT SAFE
///The number type Dnum is incorrectly marked as Send and Sync.
///This is because broccoli::Num requires Send and Sync.
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
/*
pub fn datanum_test_ret<T>(func: impl FnOnce(&mut Maker) -> T) -> (T, usize) {
    unsafe { COUNTER = 0 };
    let mut maker = Maker { _p: PhantomData };
    let k = func(&mut maker);
    (k, unsafe { COUNTER })
}*/

pub struct Maker {
    _p: PhantomData<*mut usize>, //Make it not implement send or sync
}
impl Maker {
    pub fn count(&self) -> usize {
        unsafe { COUNTER }
    }
    pub fn reset(&self) {
        unsafe { COUNTER = 0 }
    }
    pub fn build_from_rect<I: Num>(&self, rect: Rect<I>) -> Rect<Dnum<I>> {
        let ((a, b), (c, d)) = rect.get();
        Rect::new(
            Dnum(a, PhantomData),
            Dnum(b, PhantomData),
            Dnum(c, PhantomData),
            Dnum(d, PhantomData),
        )
    }
}
