use crate::pmut::PMut;
use crate::pmut::PMutPtr;

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
