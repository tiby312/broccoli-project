use crate::query::colfind::oned;
//use crate::query::colfind::ColMulti;
use super::super::tools;
use super::ColMulti;
use crate::query::inner_prelude::*;
use unchecked_unwrap::*;


pub(crate) struct DestructuredNode<'a,'b:'a, T: Aabb, AnchorAxis: Axis> {
    pub node:PMut<'a,NodeMut<'b,T>>,
    pub axis: AnchorAxis,
}

impl<'a,'b:'a,T:Aabb,AnchorAxis:Axis>  DestructuredNode<'a,'b,T,AnchorAxis>{
    #[inline(always)]
    pub fn new(axis:AnchorAxis,node:PMut<'a,NodeMut<'b,T>>)->Option<DestructuredNode<'a,'b,T,AnchorAxis>>{
        debug_assert!(node.get().div.is_some()); //TODO remove
        if node.cont.is_some(){
            Some(DestructuredNode {
                node,
                axis,
            })
        }else{
            None
        }
    }

    #[inline(always)]
    pub fn cont(&self)->&axgeom::Range<T::Num>{
        //TODO use unsafe and dont unwrap
        unsafe{
            self.node.cont.as_ref().unchecked_unwrap()
        }
    }
    /*
    #[inline(always)]
    pub fn div(&self)->N::Num{
        //TODO use unsafe and dont unwrap
        unsafe{
        self.node.get().div.unchecked_unwrap()
        }
    }
    */
}


pub(crate) struct DestructuredNodeLeaf<'a,'b:'a, T:Aabb, A: Axis> {
    pub node:PMut<'a,NodeMut<'b,T>>,
    pub axis: A,
}

impl<'a,'b:'a,T:Aabb,AnchorAxis:Axis>  DestructuredNodeLeaf<'a,'b,T,AnchorAxis>{
    #[inline(always)]
    pub fn new(axis:AnchorAxis,node:PMut<'a,NodeMut<'b,T>>)->Option<DestructuredNodeLeaf<'a,'b,T,AnchorAxis>>{
        if node.cont.is_some(){
            Some(DestructuredNodeLeaf {
                node,
                axis,
            })
        }else{
            None
        }
    }

    #[inline(always)]
    pub fn cont(&self)->&axgeom::Range<T::Num>{
        //TODO use unsafe and dont unwrap
        unsafe{
            self.node.cont.as_ref().unchecked_unwrap()
        }
    }
}



pub(crate) trait NodeHandler {
    type T: Aabb;

    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>);

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    );
}

pub(super) struct HandleNoSorted<K: ColMulti + Splitter> {
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
    fn div(&mut self) -> (Self,Self) {
        let (a,b)=self.func.div();
        (HandleNoSorted {
            func: a,
        },HandleNoSorted {
            func: b,
        })
    }
    #[inline(always)]
    fn add(&mut self,a:Self, b: Self) {
        self.func.add(a.func,b.func);
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
        mut current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.cont().intersects(anchor.cont())
        };

        if res {
            for mut a in current.node.get_range2().as_mut().iter_mut() {
                for mut b in anchor.node.get_range2().as_mut().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a.as_mut(), b.as_mut());
                    }
                }
            }
        }
    }
}

pub(super) struct HandleSorted<K: ColMulti + Splitter> {
    pub func: K,
}
impl<K: ColMulti + Splitter> HandleSorted<K> {
    #[inline(always)]
    pub fn new(a: K) -> HandleSorted<K> {
        HandleSorted {
            func: a,
        }
    }
}
impl<K: ColMulti + Splitter> Splitter for HandleSorted<K> {
    
    #[inline(always)]
    fn div(&mut self) -> (Self,Self) {
        let (a,b)=self.func.div();
        (HandleSorted {
            func: a,
        },HandleSorted {
            func: b,
        })
    }
    #[inline(always)]
    fn add(&mut self,a:Self, b: Self) {
        self.func.add(a.func,b.func);
    }
}

impl<K: ColMulti + Splitter> NodeHandler for HandleSorted<K> {
    type T = K::T;
    #[inline(always)]
    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;
        oned::find_2d(axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        if !current.axis.is_equal_to(anchor.axis) {
            let cc1=*anchor.cont();
            let cc2=*current.cont(); //TODO not egronomic
            
            //println!("{:?} {:?}",cc1,cc2);
            let ss=current.node.get_mut().bots;
            let r1 = oned::get_section_mut(anchor.axis, ss, cc1);
            let ss2=anchor.node.as_mut().get_mut().bots;
            let r2 = oned::get_section_mut(current.axis, ss2, cc2);
            
            oned::find_perp_2d1(current.axis,r1,r2, func);
        } else if current.cont().intersects(anchor.cont()) {
            oned::find_parallel_2d(
                current.axis.next(),
                current.node.get_mut().bots.as_mut(),
                anchor.node.as_mut().get_mut().bots.as_mut(),
                func,
            );
        }
    }
}
