use super::*;

///
/// Singifies this object can be swapped around in a slice
/// many times without much of a performance hit.
///
pub trait ManySwap {}

/// The underlying number type used for the tree.
/// It is auto implemented by all types that satisfy the type constraints.
/// Notice that no arithmetic is possible. The tree is constructed
/// using only comparisons and copying.
pub trait Num: PartialOrd + Copy + Default + std::fmt::Debug {}
impl<T> Num for T where T: PartialOrd + Copy + Default + std::fmt::Debug {}

///
/// Trait to signify that this object has an axis aligned bounding box.
///
pub trait Aabb {
    type Num: Num;
    fn get(&self) -> &Rect<Self::Num>;
}

/*
impl<T: Aabb> Aabb for &T {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        T::get(self)
    }
}

impl<T: Aabb> Aabb for &mut T {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        T::get(self)
    }
}
*/

impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}
impl<'a, N> ManySwap for Rect<N> {}
impl<'a, N> ManySwap for &'a mut Rect<N> {}

impl<'a, N, T> ManySwap for &'a mut (Rect<N>, T) {}

impl<'a, N, T> ManySwap for (Rect<N>, &'a mut T) {}
impl<'a, N, T> ManySwap for (Rect<N>, &'a T) {}

impl<'a, N> ManySwap for (Rect<N>, ()) {}
impl<'a, N> ManySwap for (Rect<N>, usize) {}
impl<'a, N> ManySwap for (Rect<N>, u32) {}
impl<'a, N> ManySwap for (Rect<N>, u64) {}

#[derive(Copy, Clone, Debug)]
pub struct ManySwappable<T>(pub T);
impl<T> ManySwap for ManySwappable<T> {}

impl<T> ManySwap for &mut ManySwappable<T> {}

impl<T: Aabb> Aabb for &mut ManySwappable<T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self.0.get()
    }
}
impl<T: HasInner> HasInner for &mut ManySwappable<T> {
    type Inner = T::Inner;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        self.0.destruct_mut()
    }
}

impl<T: Aabb> Aabb for ManySwappable<T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self.0.get()
    }
}

impl<T: HasInner> HasInner for ManySwappable<T> {
    type Inner = T::Inner;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        self.0.destruct_mut()
    }
}

impl<N: Num, T> Aabb for (Rect<N>, T) {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.0
    }
}

impl<N: Num, T> HasInner for (Rect<N>, T) {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0, &mut self.1)
    }
}

impl<N: Num, T> Aabb for &mut (Rect<N>, T) {
    type Num = N;
    fn get(&self) -> &Rect<Self::Num> {
        &self.0
    }
}
impl<N: Num, T> HasInner for &mut (Rect<N>, T) {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.0, &mut self.1)
    }
}

/// A bounding box container object that implements [`Aabb`] and [`HasInner`].
/// Note that `&mut BBox<N,T>` also implements [`Aabb`] and [`HasInner`].
///
/// Using this one struct the user can construct the following types for bboxes to be inserted into the tree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect)
#[derive(Debug, Copy, Clone)]
pub struct BBox<N, T> {
    pub rect: Rect<N>,
    pub inner: T,
}

impl<N, T> BBox<N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }

    pub fn many_swap(self) -> ManySwappable<Self> {
        ManySwappable(self)
    }
}

impl<'a, N> ManySwap for BBox<N, ()> {}
impl<'a, N, T> ManySwap for BBox<N, &'a mut T> {}

impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

impl<N: Num, T> HasInner for BBox<N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

impl<N: Num, T> Aabb for &mut BBox<N, T> {
    type Num = N;
    fn get(&self) -> &Rect<N> {
        &self.rect
    }
}
impl<N: Num, T> HasInner for &mut BBox<N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

///
/// BBox with a reference.
///
/// Similar to `BBox<N,&mut T>` except
/// get_inner_mut() doesnt return a `&mut &mut T`
/// but instead just a `&mut T`.
///
pub struct BBoxMut<'a, N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}
impl<'a, N, T> ManySwap for BBoxMut<'a, N, T> {}

impl<'a, N, T> BBoxMut<'a, N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: &'a mut T) -> BBoxMut<'a, N, T> {
        BBoxMut { rect, inner }
    }
}

impl<N: Num, T> Aabb for BBoxMut<'_, N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &axgeom::Rect<N> {
        &self.rect
    }
}
impl<N: Num, T> HasInner for BBoxMut<'_, N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, self.inner)
    }
}

/// When we traverse the tree in read-only mode, we can simply return a reference to each node.
/// We don't need to protect the user from only mutating parts of the BBox's since they can't
/// change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use super::*;
    use compt::Visitor;

    /// Tree Iterator that returns a protected mutable reference to each node.
    #[repr(transparent)]
    #[must_use]
    pub struct VistrMutPin<'a, N> {
        inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
    }

    impl<'a, N> VistrMutPin<'a, N> {
        #[inline(always)]
        pub(crate) fn new(
            inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        ) -> Self {
            VistrMutPin { inner }
        }
        /// It is safe to borrow the iterator and then produce mutable references from that
        /// as long as by the time the borrow ends, all the produced references also go away.
        #[inline(always)]
        pub fn borrow_mut(&mut self) -> VistrMutPin<N> {
            VistrMutPin {
                inner: self.inner.borrow_mut(),
            }
        }

        #[inline(always)]
        pub fn borrow(&self) -> Vistr<N> {
            self.inner.borrow()
        }

        #[inline(always)]
        pub fn get_height(&self) -> usize {
            compt::FixedDepthVisitor::get_height(self)
        }

        #[inline(always)]
        pub fn into_slice(self) -> AabbPin<&'a mut [N]> {
            AabbPin::new(self.inner.into_slice())
        }
    }

    impl<'a, N> compt::FixedDepthVisitor for VistrMutPin<'a, N> {}

    impl<'a, N> Visitor for VistrMutPin<'a, N> {
        type Item = AabbPin<&'a mut N>;

        #[inline(always)]
        fn next(self) -> (Self::Item, Option<[Self; 2]>) {
            let (nn, rest) = self.inner.next();

            let k = rest
                .map(|[left, right]| [VistrMutPin { inner: left }, VistrMutPin { inner: right }]);

            (AabbPin::new(nn), k)
        }

        #[inline(always)]
        fn level_remaining_hint(&self) -> (usize, Option<usize>) {
            self.inner.level_remaining_hint()
        }
    }
}
pub use vistr_mut::VistrMutPin;

pub struct Node<'a, T: Aabb> {
    /// May or may not be sorted.
    pub range: AabbPin<&'a mut [T]>,

    /// if range is empty, then value is `[default,default]`.
    /// if range is not empty, then cont is the min max bounds in on the y axis (if the node belongs to the x axis).
    pub cont: axgeom::Range<T::Num>,

    /// for non leafs:
    ///   if there is a bot either in this node or in a child node, then div is some.
    ///
    /// for leafs:
    ///   value is none
    pub div: Option<T::Num>,

    ///
    /// The minimum number of elements in a child node.
    /// If the left child has 500 bots, and the right child has 20, then
    /// this value will be 20.
    ///
    /// This is used to determine when to start a parallel task.
    /// Starting a parallel task has overhead so we only want to split
    /// one off if we know that both threads have a decent amount of work
    /// to perform in parallel.
    ///
    pub min_elem: usize,

    pub num_elem: usize,
}
impl<'a, T: Aabb> Node<'a, T> {
    pub fn borrow_range(&mut self) -> AabbPin<&mut [T]> {
        self.range.borrow_mut()
    }

    pub fn as_data(&self) -> NodeData<T::Num> {
        NodeData {
            range: self.range.len(),
            cont: self.cont,
            div: self.div,
            min_elem: self.min_elem,
            num_elem: self.num_elem,
        }
    }
}

///
/// Like [`Node`] except only has the number of elem instead of a slice..
///
#[derive(Debug, Clone)]
pub struct NodeData<N: Num> {
    pub range: usize,
    pub cont: axgeom::Range<N>,
    pub div: Option<N>,
    pub min_elem: usize,
    pub num_elem: usize,
}

pub use axgeom::Range;
pub use axgeom::Rect;
