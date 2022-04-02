
use crate::tree::halfpin::HalfPin;
pub trait CachableCollisionPairs<T>{
    fn for_every_pair(&mut self,func:impl FnMut(HalfPin<&mut T>,HalfPin<&mut T>));
}


pub struct Cacheable<'a,T,C>{
    inner:&'a mut C,
    pairs:Vec<(HalfPin<*mut T>,HalfPin<*mut T>)>
}
impl<'a,T:Aabb,C:CachableCollisionPairs<T>> Cacheable<'a,T,C>{
    pub fn new(a:&'a mut C,mut func:impl FnMut(HalfPin<&mut T>,HalfPin<&mut T>))->Self{
        let mut pairs=vec!();
        a.for_every_pair(|mut a,mut b|{
            pairs.push((a.as_ptr_mut(),b.as_ptr_mut()));
            func(a,b);
        });

        Cacheable{
            inner:a,
            pairs
        }
    }

    pub fn again(&mut self,mut func:impl FnMut(HalfPin<&mut T>,HalfPin<&mut T>)){
        for (a,b) in self.pairs.iter_mut(){
            let a=HalfPin::new(unsafe{&mut *a.as_raw()});
            let b=HalfPin::new(unsafe{&mut *b.as_raw()});
            func(a,b);
        }
    }

    pub fn finish(self)->&'a mut C{
        self.inner
    }
}
