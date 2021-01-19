//! Container trees that deref to [`Tree`]
//!
//! Most of the time using [`Tree`] is enough. But in certain cases
//! we want more control. 

use super::*;

mod tree_ind;
mod owned;
pub use self::tree_ind::*;
pub use self::owned::*;


mod dim3{
    use axgeom::*;
    use crate::node::*;
    
    pub struct BBox3D<'a,N,T>{
        pub rect:Rect<N>,
        pub inner: &'a mut (Range<N>,T)
    }

    unsafe impl<'a,N:Num,T> crate::node::Aabb for BBox3D<'a,N,T>{
        type Num=N;
        fn get(&self)->&Rect<N>{
            &self.rect
        }
    }


    
    use super::*;
    pub struct Tree3D<'a,'b,N:Num,T>{
        pub inner:Tree<'b,BBox3D<'a,N,T>>
    }


    impl<'a,'b,N:Num,T> Tree3D<'a,'b,N,T>{
        pub fn new(arr:&'a mut [BBox3D<'b,N,T>])->Tree3D<'a,'b,N,T>{
            
            let inner=Tree::new(arr);
            Tree3D{
                inner
            }
        }


        
    }
    
    

}

use alloc::boxed::Box;
