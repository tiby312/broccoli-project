use crate::query::colfind::oned;
//use crate::query::colfind::ColMulti;
use super::ColMulti;
use crate::query::inner_prelude::*;
use super::super::tools;


pub struct DestructuredNode<'a, T: Aabb, AnchorAxis: Axis> {
    pub div: &'a T::Num,
    pub range: PMut<'a, [T]>,
    pub cont: &'a axgeom::Range<T::Num>,
    pub axis: AnchorAxis,
}

pub struct DestructuredNodeLeaf<'a, T: Aabb, A: Axis> {
    pub range: PMut<'a, [T]>,
    pub cont: &'a axgeom::Range<T::Num>,
    pub axis: A,
}

pub trait NodeHandler {
    type T: Aabb;

    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>);

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: &mut DestructuredNodeLeaf<Self::T, B>,
    );
}

pub struct HandleNoSorted<K: ColMulti + Splitter> {
    pub func: K,
}
impl<K: ColMulti + Splitter> HandleNoSorted<K> {
    #[inline(always)]
    pub fn new(func: K) -> Self {
        HandleNoSorted { func }
    }
}

impl<K: ColMulti + Splitter> Splitter for HandleNoSorted<K> {
    #[inline(always)]
    fn div(&mut self) -> Self {
        HandleNoSorted {
            func: self.func.div(),
        }
    }
    #[inline(always)]
    fn add(&mut self, a: Self) {
        self.func.add(a.func);
    }
    #[inline(always)]
    fn node_start(&mut self) {
        self.func.node_start();
    }
    #[inline(always)]
    fn node_end(&mut self) {
        self.func.node_start();
    }
}

impl<K: ColMulti + Splitter> NodeHandler for HandleNoSorted<K> {
    type T = K::T;
    fn handle_node(&mut self, _axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;

        tools::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }
    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: &mut DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.cont.intersects(&anchor.cont)
        };

        if res {
            for mut a in current.range.as_mut().iter_mut() {
                for mut b in anchor.range.as_mut().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a.as_mut(), b.as_mut());
                    }
                }
            }
        }
    }
}

pub struct HandleSorted<K: ColMulti + Splitter> {
    pub sweeper: oned::Sweeper<K::T>,
    pub func: K,
}
impl<K: ColMulti + Splitter> HandleSorted<K> {
    #[inline(always)]
    pub fn new(a: K) -> HandleSorted<K> {
        HandleSorted {
            sweeper: oned::Sweeper::new(),
            func: a,
        }
    }
}
impl<K: ColMulti + Splitter> Splitter for HandleSorted<K> {
    #[inline(always)]
    fn div(&mut self) -> Self {
        HandleSorted {
            sweeper: oned::Sweeper::new(),
            func: self.func.div(),
        }
    }
    #[inline(always)]
    fn add(&mut self, a: Self) {
        self.func.add(a.func);
    }
    #[inline(always)]
    fn node_start(&mut self) {
        self.func.node_start();
    }
    #[inline(always)]
    fn node_end(&mut self) {
        self.func.node_start();
    }
}

impl<K: ColMulti + Splitter> NodeHandler for HandleSorted<K> {
    type T = K::T;
    #[inline(always)]
    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;
        self.sweeper.find_2d(axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: &mut DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        if !current.axis.is_equal_to(anchor.axis) {
            let r1 = oned::get_section_mut(anchor.axis, current.range.as_mut(), anchor.cont);
            let r2 = oned::get_section_mut(current.axis, anchor.range.as_mut(), current.cont);
            self.sweeper.find_perp_2d1(anchor.axis, r1, r2, func);
        } else if current.cont.intersects(anchor.cont) {
            self.sweeper.find_parallel_2d(
                current.axis.next(),
                current.range.as_mut(),
                anchor.range.as_mut(),
                func,
            );
        }
    }
}
