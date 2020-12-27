use super::*;
use super::raycast::*;
use super::knearest::*;
///Provides the naive implementation of the [`Tree`] api.
pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a, [T]>,
}


impl<'a,T:Aabb> NaiveQueries for NaiveAlgs<'a,T>{
    type T=T;
    type Num=T::Num;
    fn get_slice_mut(&mut self)->PMut<[T]>{
        self.bots.borrow_mut()
    }

}



impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn from_slice(a: &'a mut [T]) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots: PMut::new(a) }
    }
    #[must_use]
    pub fn new(bots: PMut<'a, [T]>) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots }
    }

    //#[cfg(feature = "nbody")]
    pub fn nbody(&mut self, func: impl FnMut(PMut<T>, PMut<T>)) {
        nbody::naive_mut(self.bots.borrow_mut(), func);
    }
}
