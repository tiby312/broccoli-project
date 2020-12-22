use crate::pmut::PMut;
use crate::node::Aabb;

pub(crate) fn for_every_pair<T: Aabb>(mut arr: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
    loop {
        let temp = arr;
        match temp.split_first_mut() {
            Some((mut b1, mut x)) => {
                for mut b2 in x.borrow_mut().iter_mut() {
                    func(b1.borrow_mut(), b2.borrow_mut());
                }
                arr = x;
            }
            None => break,
        }
    }
}



///Two unordered vecs backed by one vec.
///Pushing and retaining from the first cec,
///can change the ordering of the second vec.
///Assume both vecs ordering can change at any time.
pub struct TwoUnorderedVecs<T>{
    inner:Vec<T>,
    first_length:usize
}

impl<T> TwoUnorderedVecs<T>{

    pub fn get_first_mut(&mut self)->&mut [T]{
        &mut self.inner[..self.first_length]
    }
    pub fn get_second_mut(&mut self)->&mut [T]{
        &mut self.inner[self.first_length..]
    }
    pub fn push_first(&mut self,a:T){
        let len=self.inner.len();
        self.inner.push(a); 
        //now len is actual one less than current length.
        self.inner.swap(self.first_length,len);
        self.first_length+=1;
    }
    pub fn push_second(&mut self,b:T){
        self.inner.push(b);
    }

    pub fn truncate_first(&mut self,num:usize){
        let total_len=self.inner.len();

        //the number to be removed
        let diff=self.first_length-num;

        self.first_length=num;


        for a in (0..diff).rev(){
            self.inner.swap(self.first_length-a,total_len-a);
        }

        self.inner.truncate(total_len-diff);
    }
    pub fn truncate_second(&mut self,num:usize){
        self.inner.truncate(num);
    }

    pub fn retain_first_mut(&mut self,mut func:impl FnMut(&mut T)->bool){
        let len = self.get_first_mut().len();
        let mut del = 0;
        {
            //let v = &mut **self;
            let v= self.get_first_mut();

            let mut cursor = 0;
            for _ in 0..len {
                if !func(&mut v[cursor]) {
                    v.swap(cursor, len - 1 - del);
                    del += 1;
                } else {
                    cursor += 1;
                }
            }
        }
        if del > 0 {
            self.truncate_first(len - del);
        }
    }

    pub fn retain_second_mut(&mut self,mut func:impl FnMut(&mut T)->bool){
        let len = self.get_second_mut().len();
        let mut del = 0;
        {
            //let v = &mut **self;
            let v= self.get_second_mut();

            let mut cursor = 0;
            for _ in 0..len {
                if !func(&mut v[cursor]) {
                    v.swap(cursor, len - 1 - del);
                    del += 1;
                } else {
                    cursor += 1;
                }
            }
        }
        if del > 0 {
            self.truncate_second(len - del);
        }
    }
}


pub trait RetainMutUnordered<T> {
    fn retain_mut_unordered<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

use alloc::vec::Vec;
impl<T> RetainMutUnordered<T> for Vec<T> {
    //TODO remove this inline?
    #[inline(always)]
    fn retain_mut_unordered<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            let mut cursor = 0;
            for _ in 0..len {
                if !f(&mut v[cursor]) {
                    v.swap(cursor, len - 1 - del);
                    del += 1;
                } else {
                    cursor += 1;
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
