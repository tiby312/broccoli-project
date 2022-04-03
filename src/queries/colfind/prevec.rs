use crate::halfpin::HalfPin;
use alloc::vec::Vec;

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec {
    vec: Vec<usize>,
}

impl Default for PreVec {
    fn default() -> Self {
        PreVec::new()
    }
}

impl PreVec {
    #[allow(dead_code)]
    #[inline(always)]
    pub fn new() -> PreVec {
        PreVec { vec: Vec::new() }
    }
    #[inline(always)]
    pub fn with_capacity(num: usize) -> PreVec {
        PreVec {
            vec: Vec::with_capacity(num),
        }
    }

    ///Take advantage of the big capacity of the original vec.
    pub fn extract_vec<'a, 'b, T>(&'a mut self) -> Vec<HalfPin<&'b mut T>> {
        let mut v = Vec::new();
        core::mem::swap(&mut v, &mut self.vec);
        revec::convert_empty_vec(v)
    }

    ///Return the big capacity vec
    pub fn insert_vec<T>(&mut self, vec: Vec<HalfPin<&'_ mut T>>) {
        let mut v = revec::convert_empty_vec(vec);
        core::mem::swap(&mut self.vec, &mut v)
    }
}
