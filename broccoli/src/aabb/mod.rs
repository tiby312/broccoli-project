//!
//! Contains AABB primitives and building blocks.
//!

pub mod pin;

use super::*;
use pin::*;

pub use axgeom::Range;
pub use axgeom::Rect;

///
/// Singifies this object can be swapped around in a slice
/// many times without much of a performance hit.
///
pub trait ManySwap {}

impl<N> ManySwap for Rect<N> {}
impl<'a, N> ManySwap for &'a mut Rect<N> {}

impl<'a, N, T> ManySwap for &'a mut (Rect<N>, T) {}

impl<'a, N, T> ManySwap for (Rect<N>, &'a mut T) {}
impl<'a, N, T> ManySwap for (Rect<N>, &'a T) {}

impl<N> ManySwap for (Rect<N>, ()) {}
impl<N> ManySwap for (Rect<N>, usize) {}
impl<N> ManySwap for (Rect<N>, u32) {}
impl<N> ManySwap for (Rect<N>, u64) {}

///
/// Wrapper to opt in to being allowed to be fed to swap intensive algorithms.
///
#[derive(Copy, Clone, Debug)]
pub struct ManySwappable<T>(pub T);
impl<T> ManySwap for ManySwappable<T> {}

impl<T> ManySwap for &mut ManySwappable<T> {}

impl<T: Aabb> Aabb for &mut ManySwappable<T> {
    type Num = T::Num;

    fn minx(&self)->&Self::Num{
        self.0.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.0.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.0.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.0.maxy()
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

    fn minx(&self)->&Self::Num{
        self.0.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.0.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.0.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.0.maxy()
    }
}

impl<T: HasInner> HasInner for ManySwappable<T> {
    type Inner = T::Inner;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        self.0.destruct_mut()
    }
}

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
    // fn get(&self) -> &Rect<Self::Num>;
    // fn xrange(&self) -> [&Self::Num; 2] {
    //     let a = &self.get().get_range(XAXIS).start;
    //     let b = &self.get().get_range(XAXIS).start;
    //     [a, b]
    // }
    // fn yrange(&self) -> [&Self::Num; 2] {
    //     let a = &self.get().get_range(YAXIS).start;
    //     let b = &self.get().get_range(YAXIS).start;
    //     [a, b]
    // }
    fn minx(&self)->&Self::Num;
    fn maxx(&self)->&Self::Num;
    fn miny(&self)->&Self::Num;
    fn maxy(&self)->&Self::Num;
}

pub(crate) trait AabbExt: Aabb {
    fn xrange(&self) -> [&Self::Num; 2]{
        [self.minx(),self.maxx()]
    }
    fn yrange(&self)->[&Self::Num;2]{
        [self.miny(),self.maxy()]
    }

    fn to_range(&self, axis: impl Axis) -> Range2<Self::Num> {
        if axis.is_xaxis() {
            Range2(self.xrange())
        } else {
            Range2(self.yrange())
        }
    }

    fn intersects_aabb(&self, other: &impl Aabb<Num = Self::Num>) -> bool {
        self.to_range(XAXIS).intersects(other.to_range(XAXIS))
            && self.to_range(YAXIS).intersects(other.to_range(YAXIS))
    }

    fn contains_aabb(&self, other: &impl Aabb<Num = Self::Num>) -> bool {
        self.to_range(XAXIS).contains_range(&other.to_range(XAXIS))
            && self.to_range(YAXIS).contains_range(&other.to_range(YAXIS))
    }
    fn contains_point(&self, point: &Vec2<Self::Num>) -> bool {
        self.to_range(XAXIS).contains(&point.x) && self.to_range(YAXIS).contains(&point.y)
    }

    fn make_rect(&self) -> axgeom::Rect<Self::Num> {
        let [a, b] = self.xrange();
        let [c, d] = self.yrange();
        rect(*a, *b, *c, *d)
    }

    // fn start_axis(&self,axis:impl Axis)->&Self::Num{
    //     self.to_range(axis).0[0]
    // }
    // fn end_axis(&self,axis:impl Axis)->&Self::Num{
    //     self.to_range(axis).0[1]
    // }
}
impl<T: Aabb> AabbExt for T {}

#[derive(Copy, Clone)]
pub(crate) struct Range2<'a, N>([&'a N; 2]);

impl<'a, N> Range2<'a, N> {
    pub fn start(&self) -> &'a N {
        self.0[0]
    }
    pub fn end(&self) -> &'a N {
        self.0[1]
    }

    pub fn from_range(range: &'a Range<N>) -> Self {
        Range2([&range.start, &range.end])
    }

    pub fn contains_ext(&self, pos: &N) -> std::cmp::Ordering
    where
        N: PartialOrd,
    {
        if pos < self.start() {
            core::cmp::Ordering::Less
        } else if pos > self.end() {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }

    ///Returns true if self contains the specified range.
    #[inline(always)]
    pub fn contains_range(&self, val: &Range2<N>) -> bool
    where
        N: PartialOrd,
    {
        self.start() <= val.start() && val.end() <= self.end()
    }

    pub fn intersects(self, val: Range2<'a, N>) -> bool
    where
        N: PartialOrd,
    {
        !(self.end() < val.start() || val.end() < self.start())
    }

    pub fn contains(self, pos: &N) -> bool
    where
        N: PartialOrd,
    {
        !(pos < self.start() || pos > self.end())
    }
}

impl<N: Num> Aabb for Rect<N> {
    type Num = N;

    fn minx(&self)->&Self::Num{
        &self.x.start
    }
    fn maxx(&self)->&Self::Num{
        &self.x.end
    }
    fn miny(&self)->&Self::Num{
        &self.y.start
    }
    fn maxy(&self)->&Self::Num{
        &self.y.end
    }
}

impl<N: Num, T> Aabb for (Rect<N>, T) {
    type Num = N;
    
    fn minx(&self)->&Self::Num{
        self.0.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.0.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.0.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.0.maxy()
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
    
    fn minx(&self)->&Self::Num{
        self.0.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.0.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.0.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.0.maxy()
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

impl<N> ManySwap for BBox<N, ()> {}
impl<'a, N, T> ManySwap for BBox<N, &'a mut T> {}

impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    
    fn minx(&self)->&Self::Num{
        self.rect.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.rect.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.rect.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.rect.maxy()
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
    
    fn minx(&self)->&Self::Num{
        self.rect.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.rect.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.rect.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.rect.maxy()
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
    
    fn minx(&self)->&Self::Num{
        self.rect.minx()
    }
    fn maxx(&self)->&Self::Num{
        self.rect.maxx()
    }
    fn miny(&self)->&Self::Num{
        self.rect.miny()
    }
    fn maxy(&self)->&Self::Num{
        self.rect.maxy()
    }
}
impl<N: Num, T> HasInner for BBoxMut<'_, N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, self.inner)
    }
}
