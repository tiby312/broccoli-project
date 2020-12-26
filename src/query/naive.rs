use super::*;
///Provides the naive implementation of the [`Tree`] api.
pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a, [T]>,
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    /*
    #[must_use]
    pub fn raycast_mut<Acc>(
        &mut self,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        border: Rect<T::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<T>>, T::Num)> {
        let mut rtrait = raycast::RayCastClosure {
            a: start,
            broad,
            fine,
            _p: PhantomData,
        };
        raycast::raycast_naive_mut(self.bots.borrow_mut(), ray, &mut rtrait, border)
    }
    */
    pub fn raycast_mut(&mut self,ray:axgeom::Ray<T::Num>,rtrait: &mut impl crate::query::RayCast<T=T,N=T::Num>)-> axgeom::CastResult<(Vec<PMut<T>>, T::Num)>{
        raycast::raycast_naive_mut(self.bots.borrow_mut(), ray, rtrait)
    }


    pub fn k_nearest_mut<'b, K: query::Knearest<T = T, N = T::Num>>(
        &'b mut self,
        point: Vec2<T::Num>,
        num: usize,
        ktrait: &mut K
    ) -> KResult<T>
    where
        'a: 'b,
    {
        k_nearest::k_nearest_naive_mut(self.bots.borrow_mut(), point, num, ktrait)
    }
    /*
    #[must_use]
    pub fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<T::Num>,
        num: usize,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
    ) -> KResult<T> {
        let mut knear = k_nearest::KnearestClosure {
            acc: start,
            broad,
            fine,
            _p: PhantomData,
        };
        k_nearest::k_nearest_naive_mut(self.bots.borrow_mut(), point, num, &mut knear)
    }
    */
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    pub fn for_all_in_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_in_rect_mut(self.bots.borrow_mut(), rect, func);
    }
    pub fn for_all_not_in_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_not_in_rect_mut(self.bots.borrow_mut(), rect, func);
    }

    pub fn for_all_intersect_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_intersect_rect_mut(self.bots.borrow_mut(), rect, func);
    }

    pub fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        colfind::query_naive_mut(self.bots.borrow_mut(), |a, b| func(a, b));
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
