pub unsafe trait TrustedCollisionPairs<T> {
    fn for_every_pair(&mut self, func: impl FnMut(&mut T, &mut T));
}

pub struct Cacheable<'a, T, C> {
    inner: &'a mut C,
    pairs: Vec<(*mut T, *mut T)>,
}
impl<'a, T, C: TrustedCollisionPairs<T>> Cacheable<'a, T, C> {
    pub fn new(a: &'a mut C) -> Self {
        Self::with_func(a, |_, _| {})
    }

    pub fn with_func(a: &'a mut C, mut func: impl FnMut(&mut T, &mut T)) -> Self {
        let mut pairs = vec![];
        a.for_every_pair(|a, b| {
            pairs.push((a as *mut _, b as *mut _));
            func(a, b);
        });

        Cacheable { inner: a, pairs }
    }

    pub fn again(&mut self, mut func: impl FnMut(&mut T, &mut T)) {
        for (a, b) in self.pairs.iter_mut() {
            let a = unsafe { &mut **a };
            let b = unsafe { &mut **b };
            func(a, b);
        }
    }

    pub fn finish(self) -> &'a mut C {
        self.inner
    }
}
