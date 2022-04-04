
pub trait EveryPair<T> {
    fn every_pair(&mut self, func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>));
}

pub struct PairCollector<'a, T, S: EveryPair<T>> {
    inner: &'a mut S,
    cached: Option<Vec<[HalfPin<*mut T>; 2]>>,
}
impl<'a, T, S: EveryPair<T>> EveryPair<T> for PairCollector<'a, T, S> {
    fn every_pair(&mut self, mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>)) {
        //cache results
        if let Some(v)=self.cached.as_mut(){
            for foo in v.iter_mut(){
                let a=HalfPin::new(unsafe{&mut *foo[0].clone().into_raw()});
                let b=HalfPin::new(unsafe{&mut *foo[1].clone().into_raw()});
                func(a,b)
            }
        }else{
            let mut v=vec!();
            self.inner.every_pair(|mut a,mut b|{
                func(a.borrow_mut(),b.borrow_mut());
                v.push([a.into_ptr(),b.into_ptr()]);
            });
            self.cached=Some(v);
        }
    }
}
