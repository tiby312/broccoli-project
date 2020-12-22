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



//TODO put in module
#[test]
fn test_push(){
    let mut k=TwoUnorderedVecs::new();
    k.push_first(9);    
    k.push_second(0);
    k.push_first(3);
    
    k.push_first(6);
    k.push_second(8);
    k.push_first(5);
    
    slices_match(k.get_first_mut(),&mut [9,3,6,5]);
    slices_match(k.get_second_mut(),&mut [0,8]);

    assert_eq!(k.first_length,4);

    k.truncate_first(2);
    k.truncate_second(1);


    
    slices_match(k.get_first_mut(),&mut [3,9]);
    slices_match(k.get_second_mut(),&mut [8]);
    
    assert_eq!(k.get_first_mut().len(),2);
    assert_eq!(k.get_second_mut().len(),1);
    assert_eq!(k.first_length,2);

    k.push_first(4);
    k.push_first(6);
    k.push_first(7);
    k.push_first(8);

    k.push_second(7);
    k.push_second(3);
    k.push_second(2);
    k.push_second(4);
    

    k.retain_first_mut_unordered(|&mut a|a%2==1);
    k.retain_second_mut_unordered(|&mut a|a%2==0);
    
    dbg!(k.get_first_mut());
    dbg!(k.get_second_mut());

    slices_match(k.get_first_mut(),&mut [9,3,7]);
    slices_match(k.get_second_mut(),&mut [8,2,4]);
}

fn slices_match<T:Eq>(arr1:&[T],arr2:&[T]){
    for a in arr2.iter(){
        assert!(arr1.contains(a));
    }
    for b in arr1.iter(){
        assert!(arr2.contains(b));
    }
    assert_eq!(arr1.len(),arr2.len());
}

///Two unordered vecs backed by one vec.
///Pushing and retaining from the first cec,
///can change the ordering of the second vec.
///Assume both vecs ordering can change at any time.
#[derive(Debug)]
pub struct TwoUnorderedVecs<T>{
    inner:Vec<T>,
    first_length:usize
}

impl<T> TwoUnorderedVecs<T>{
    pub fn new()->Self{
        TwoUnorderedVecs{inner:Vec::new(),first_length:0}
    }
    pub fn get_first_mut(&mut self)->&mut [T]{
        &mut self.inner[..self.first_length]
    }
    pub fn get_second_mut(&mut self)->&mut [T]{
        &mut self.inner[self.first_length..]
    }

    //    [----------][---------]
    pub fn push_first(&mut self,a:T){
        
        let total_len=self.inner.len();

        self.inner.push(a); 
        
        //now len is actually one less than current length.
        //if self.first_length>0{
            self.inner.swap(self.first_length,total_len);
        //}
        self.first_length+=1;
    }
    pub fn push_second(&mut self,b:T){
        self.inner.push(b);
    }


    //    [----------][---------]   
    //       5              8        13
    //     truncate_Frist(2)
    pub fn truncate_first(&mut self,num:usize){
        let total_len=self.inner.len();

        //the number to be removed
        let diff=self.first_length-num;


        for a in (0..diff).rev(){
            self.inner.swap(self.first_length-a-1,total_len-a-1);
        }

        self.first_length=num;

        self.inner.truncate(total_len-diff);

    }

    pub fn truncate_second(&mut self,num:usize){
        self.inner.truncate(self.first_length+num);
    }

    pub fn retain_first_mut_unordered(&mut self,mut func:impl FnMut(&mut T)->bool){
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

    pub fn retain_second_mut_unordered(&mut self,mut func:impl FnMut(&mut T)->bool){
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
