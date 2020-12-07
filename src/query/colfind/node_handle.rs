use crate::query::colfind::oned;
//use crate::query::colfind::ColMulti;
use super::super::tools;
use super::ColMulti;
use crate::query::inner_prelude::*;

pub(crate) struct DestructuredNode<'a, N: Node, AnchorAxis: Axis> {
    pub node:PMut<'a,N>,
    pub axis: AnchorAxis,
}

impl<'a,N:Node,AnchorAxis:Axis>  DestructuredNode<'a,N,AnchorAxis>{
    pub fn new(axis:AnchorAxis,node:PMut<'a,N>)->Option<DestructuredNode<'a,N,AnchorAxis>>{
        debug_assert!(node.get().div.is_some()); //TODO remove
        if node.get().cont.is_some(){
            Some(DestructuredNode {
                node,
                axis,
            })
        }else{
            None
        }
    }

    pub fn cont(&self)->&axgeom::Range<N::Num>{
        //TODO use unsafe and dont unwrap
        self.node.get().cont.as_ref().unwrap()
    }
    pub fn div(&self)->N::Num{
        //TODO use unsafe and dont unwrap
        self.node.get().div.unwrap()
    }
}


pub(crate) struct DestructuredNodeLeaf<'a, N:Node, A: Axis> {
    pub node:PMut<'a,N>,
    pub axis: A,
}

impl<'a,N:Node,AnchorAxis:Axis>  DestructuredNodeLeaf<'a,N,AnchorAxis>{
    pub fn new(axis:AnchorAxis,node:PMut<'a,N>)->Option<DestructuredNodeLeaf<'a,N,AnchorAxis>>{
        debug_assert!(node.get().div.is_some()); //TODO remove
        if node.get().cont.is_some(){
            Some(DestructuredNodeLeaf {
                node,
                axis,
            })
        }else{
            None
        }
    }
    pub fn cont(&self)->&axgeom::Range<N::Num>{
        //TODO use unsafe and dont unwrap
        self.node.get().cont.as_ref().unwrap()
    }
}



pub(crate) trait NodeHandler {
    type T: Aabb;

    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>);

    fn handle_children<A: Axis, B: Axis,N:Node<T=Self::T,Num=<Self::T as Aabb>::Num>>(
        &mut self,
        anchor: &mut DestructuredNode<N, A>,
        current: DestructuredNodeLeaf<N, B>,
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
    fn handle_children<A: Axis, B: Axis,N:Node<T=Self::T,Num=<Self::T as Aabb>::Num>>(
        &mut self,
        anchor: &mut DestructuredNode<N, A>,
        mut current: DestructuredNodeLeaf<N, B>,
    ) {
        let func = &mut self.func;

        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.cont().intersects(anchor.cont())
        };

        if res {
            //TODO too many accessors
            for mut a in current.node.as_mut().get_mut().bots.as_mut().iter_mut() {
                for mut b in anchor.node.as_mut().get_mut().bots.as_mut().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a.as_mut(), b.as_mut());
                    }
                }
            }
        }
    }
}

pub(super) struct HandleSorted<K: ColMulti + Splitter> {
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
    fn div(&mut self) -> (Self,Self) {
        let (a,b)=self.func.div();
        (HandleSorted {
            sweeper: oned::Sweeper::new(),
            func: a,
        },HandleSorted {
            sweeper: oned::Sweeper::new(),
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
        self.sweeper.find_2d(axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis,N:Node<T=Self::T,Num=<Self::T as Aabb>::Num>>(
        &mut self,
        anchor: &mut DestructuredNode<N, A>,
        current: DestructuredNodeLeaf<N, B>,
    ) {
        let func = &mut self.func;

        if !current.axis.is_equal_to(anchor.axis) {
            let cc1=*anchor.cont();
            let cc2=*current.cont(); //TODO not egronomic
            
            let ss=current.node.get_mut().bots;
            let r1 = oned::get_section_mut(anchor.axis, ss, cc1);
            let ss2=anchor.node.as_mut().get_mut().bots;
            let r2 = oned::get_section_mut(current.axis, ss2, cc2);
            self.sweeper.find_perp_2d1(anchor.axis, r1, r2, func);
        } else if current.cont().intersects(anchor.cont()) {
            self.sweeper.find_parallel_2d(
                current.axis.next(),
                current.node.get_mut().bots.as_mut(),
                anchor.node.as_mut().get_mut().bots.as_mut(),
                func,
            );
        }
    }
}
