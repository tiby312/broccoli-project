use crate::inner_prelude::*;
use crate::query::*;


struct KnearestBBoxMutInt32<T>{
    _p:PhantomData<T>
}
impl<T> KnearestBBoxMutInt32<T>{
    pub fn new()->Self{
        KnearestBBoxMutInt32{_p:PhantomData}
    }
}
impl<T> Knearest for KnearestBBoxMutInt32<T>{
    type T = BBoxMut<i32,T>;
    type N = i32;
   
    fn distance_to_rect(&self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
        rect.distance_squared_to_point(point)
    }
}