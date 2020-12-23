
use crate::inner_prelude::*;

pub use axgeom::Range;
pub use axgeom::Rect;

///The underlying number type used for the tree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait Num: PartialOrd + Copy{}
impl<T> Num for T where T: PartialOrd + Copy{}

///Trait to signify that this object has an axis aligned bounding box.
///[`Aabb::get()`] must return a aabb with the same value in it while the element
///is in the tree. This is hard for the user not to do, this the user
///does not have `&mut self`,
///but it is still possible through the use of static objects or `RefCell` / `Mutex`, etc.
///Using these type of methods the user could make different calls to get()
///return different aabbs.
///This is unsafe since we allow query algorithms to assume the following:
///If two object's aabb's don't intersect, then they can be mutated at the same time.
pub unsafe trait Aabb {
    type Num: Num;
    fn get(&self) -> &Rect<Self::Num>;
}

unsafe impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}


///A bounding box container object that implements [`Aabb`] and [`HasInner`].
///Note that `&mut BBox<N,T>` also implements [`Aabb`] and [`HasInner`].
///
///Using this one struct the user can construct the following types for bboxes to be inserted into the tree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BBox<N, T> {
    pub rect: Rect<N>,
    pub inner: T,
}

impl<N, T> BBox<N, T> {
    ///Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}

use core::convert::TryFrom;
impl<N: Copy, T> BBox<N, T> {
    ///Creates a `(Rect<N>,&mut T)` from a `(Rect<N>,T)`
    #[inline(always)]
    #[must_use]
    pub fn into_semi_direct(&mut self) -> BBox<N, &mut T> {
        BBox {
            rect: self.rect.clone(),
            inner: &mut self.inner,
        }
    }

    ///Simply returns a mutable reference
    #[inline(always)]
    #[must_use]
    pub fn into_indirect(&mut self) -> &mut BBox<N, T> {
        self
    }

    ///Change the number type of the Rect using
    ///promitive cast.
    #[inline(always)]
    #[must_use]
    pub fn inner_as<B: 'static + Copy>(self) -> BBox<B, T>
    where
        N: num_traits::AsPrimitive<B>,
    {
        BBox {
            rect: self.rect.inner_as(),
            inner: self.inner,
        }
    }

    ///Change the number type using `From`
    #[inline(always)]
    #[must_use]
    pub fn inner_into<A: From<N>>(self) -> BBox<A, T> {
        BBox {
            rect: self.rect.inner_into(),
            inner: self.inner,
        }
    }

    ///Change the number type using `TryFrom`
    #[inline(always)]
    #[must_use]
    pub fn inner_try_into<A: TryFrom<N>>(self) -> Result<BBox<A, T>, A::Error> {
        Ok(BBox {
            rect: self.rect.inner_try_into()?,
            inner: self.inner,
        })
    }
}

unsafe impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

unsafe impl<N: Num, T> Aabb for &mut BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for &mut BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}


///When we traverse the tree in read-only mode, we can simply return a reference to each node.
///We don't need to protect the user from only mutating parts of the BBox's since they can't
///change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use crate::inner_prelude::*;

    /// Tree Iterator that returns a protected mutable reference to each node.
    #[repr(transparent)]
    pub struct VistrMut<'a, N> {
        pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
    }

    impl<'a, N> VistrMut<'a, N> {
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        #[inline(always)]
        pub fn borrow_mut(&mut self) -> VistrMut<N> {
            VistrMut {
                inner: self.inner.borrow_mut(),
            }
        }

        #[inline(always)]
        pub fn into_slice(self) -> PMut<'a, [N]> {
            PMut::new(self.inner.into_slice())
        }
    }

    impl<'a, N> core::ops::Deref for VistrMut<'a, N> {
        type Target = Vistr<'a, N>;

        #[inline(always)]
        fn deref(&self) -> &Vistr<'a, N> {
            unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
        }
    }

    unsafe impl<'a, N> compt::FixedDepthVisitor for VistrMut<'a, N> {}

    impl<'a, N> Visitor for VistrMut<'a, N> {
        type Item = PMut<'a, N>;

        #[inline(always)]
        fn next(self) -> (Self::Item, Option<[Self; 2]>) {
            let (nn, rest) = self.inner.next();

            let k = match rest {
                Some([left, right]) => Some([VistrMut { inner: left }, VistrMut { inner: right }]),
                None => None,
            };
            (PMut::new(nn), k)
        }

        #[inline(always)]
        fn level_remaining_hint(&self) -> (usize, Option<usize>) {
            self.inner.level_remaining_hint()
        }

        #[inline(always)]
        fn dfs_preorder(self, mut func: impl FnMut(Self::Item)) {
            self.inner.dfs_preorder(move |a| func(PMut::new(a)));
        }
    }
}
pub use vistr_mut::VistrMut;

///A Node in a Tree.
#[repr(C)]
pub(crate) struct NodePtr<T: Aabb> {
    _range: PMutPtr<[T]>,

    //range is empty iff cont is none.
    _cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    _div: Option<T::Num>,
}

impl<'a, T: Aabb> AsRef<NodePtr<T>> for Node<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &NodePtr<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

///A node in [`Tree`].
#[repr(C)]
pub struct Node<'a, T: Aabb> {
    pub range: PMut<'a, [T]>,
    //range is empty iff cont is none.
    pub cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some if either this node as bots or a child does
    //for leafs:
    //  div is none
    pub div: Option<T::Num>,
}
