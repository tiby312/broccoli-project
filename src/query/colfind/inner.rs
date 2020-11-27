use super::node_handle::*;
use super::*;
use crate::inner_prelude::*;

struct GoDownRecurser<'a, N: Node, NN: NodeHandler<T = N::T>, B: Axis> {
    anchor: DestructuredNode<'a, N::T, B>,
    sweeper: &'a mut NN,
}

impl<'a, N: Node, NN: NodeHandler<T = N::T>, B: Axis> GoDownRecurser<'a, N, NN, B> {
    #[inline(always)]
    fn new(
        anchor: DestructuredNode<'a, N::T, B>,
        sweeper: &'a mut NN,
    ) -> GoDownRecurser<'a, N, NN, B> {
        GoDownRecurser { anchor, sweeper }
    }

    fn go_down<
        A: Axis, //this axis
    >(
        &mut self,
        this_axis: A,
        m: VistrMut<N>,
    ) {
        let anchor_axis = self.anchor.axis;
        let (nn, rest) = m.next();
        let nn = nn.get_mut();
        match rest {
            Some([left, right]) => {
                let div = match nn.div {
                    Some(d) => d,
                    None => return,
                };

                if let Some(cont) = nn.cont {
                    let mut current = DestructuredNodeLeaf {
                        axis: this_axis,
                        range: nn.bots,
                        cont,
                    };
                    self.sweeper.handle_children(&mut self.anchor, &mut current);
                }

                if this_axis.is_equal_to(anchor_axis) {
                    if *div >= self.anchor.cont.start {
                        self.go_down(this_axis.next(), left);
                    }

                    if *div <= self.anchor.cont.end {
                        self.go_down(this_axis.next(), right);
                    };
                } else {
                    self.go_down(this_axis.next(), left);
                    self.go_down(this_axis.next(), right);
                }
            }
            None => {
                if let Some(cont) = nn.cont {
                    let mut current = DestructuredNodeLeaf {
                        axis: this_axis,
                        range: nn.bots,
                        cont,
                    };
                    self.sweeper.handle_children(&mut self.anchor, &mut current);
                }
            }
        }
    }
}

pub(crate) struct ColFindRecurser<N: Node, K: Splitter, S: NodeHandler<T = N::T> + Splitter> {
    _p: PhantomData<(N, K, S)>,
}
impl<
        N: Node + Send + Sync,
        K: Splitter + Send + Sync,
        S: NodeHandler<T = N::T> + Splitter + Send + Sync,
    > ColFindRecurser<N, K, S>
{
    pub fn recurse_par<A: Axis, JJ: par::Joiner>(
        &self,
        this_axis: A,
        par: JJ,
        sweeper: &mut S,
        m: VistrMut<N>,
        splitter: &mut K,
    ) {
       
        let (nn, rest) = m.next();
        let mut nn = nn.get_mut();
        
        match rest {
            Some([mut left, mut right]) => {
                let div = match nn.div {
                    Some(d) => d,
                    None => {
                        return;
                    }
                };

                let (mut splitter11,mut splitter22) = splitter.div();
                
                sweeper.handle_node(this_axis.next(), nn.bots.as_mut());


                if let Some(cont) = nn.cont {
                    let nn = DestructuredNode {
                        range: nn.bots,
                        cont,
                        div,
                        axis: this_axis,
                    };

                    let left = left.create_wrap_mut();
                    let right = right.create_wrap_mut();
                    let mut g = GoDownRecurser::new(nn, sweeper);
                    g.go_down(this_axis.next(), left);
                    g.go_down(this_axis.next(), right);
                }


                match par.next() {
                    par::ParResult::Parallel([dleft, dright]) => {
                        let (mut sweeper1,mut sweeper2) = sweeper.div();
                        let (splitter11ref,splitter22ref)=(&mut splitter11,&mut splitter22);
                        let (sweeper11ref,sweeper22ref)=(&mut sweeper1,&mut sweeper2);

                        
                        let af = move || {
                            self.recurse_par(
                                this_axis.next(),
                                dleft,
                                sweeper11ref,
                                left,
                                splitter11ref,
                            )
                        };
                        let bf = move || {
                            self.recurse_par(
                                this_axis.next(),
                                dright,
                                sweeper22ref,
                                right,
                                splitter22ref,
                            )
                        };
                        rayon::join(af, bf);
                    
                        sweeper.add(sweeper1,sweeper2);
                    }
                    par::ParResult::Sequential(_) => {
                        self.recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
                        self.recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);
                    }
                }
            
                splitter.add(splitter11,splitter22);
            }
            None => {
                sweeper.leaf_start();
                splitter.leaf_start();
                sweeper.handle_node(this_axis.next(), nn.bots.as_mut());
                sweeper.leaf_end();
                splitter.leaf_end();
            }
        }
    }
}

impl<N: Node, K: Splitter, S: NodeHandler<T = N::T> + Splitter> ColFindRecurser<N, K, S> {
    #[inline(always)]
    pub fn new() -> ColFindRecurser<N, K, S> {
        ColFindRecurser { _p: PhantomData }
    }

    pub fn recurse_seq<A: Axis>(
        &self,
        this_axis: A,
        sweeper: &mut S,
        m: VistrMut<N>,
        splitter: &mut K,
    ) {

        let (nn, rest) = m.next();
        let mut nn = nn.get_mut();

        
        match rest {
            Some([mut left, mut right]) => {
                
                let (mut splitter11,mut splitter22) = splitter.div();
                
                //Continue to recurse even if we know there are no more bots
                //This simplifies query algorithms that might be building up 
                //a tree.
                if let Some(div)=nn.div{
                    sweeper.handle_node(this_axis.next(), nn.bots.as_mut());


                    if let Some(cont) = nn.cont {
                        let nn = DestructuredNode {
                            range: nn.bots,
                            cont,
                            div,
                            axis: this_axis,
                        };

                        let left = left.create_wrap_mut();
                        let right = right.create_wrap_mut();
                        let mut g = GoDownRecurser::new(nn, sweeper);
                        g.go_down(this_axis.next(), left);
                        g.go_down(this_axis.next(), right);
                    }
                }


                self.recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
                self.recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);
            
                splitter.add(splitter11,splitter22);
            }
            None => {
                sweeper.leaf_start();
                splitter.leaf_start();
                sweeper.handle_node(this_axis.next(), nn.bots.as_mut());
                sweeper.leaf_end();
                splitter.leaf_end();
            }
        }
    }
}

pub(super) struct QueryFnMut<T, F>(F, PhantomData<T>);
impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> QueryFnMut<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFnMut<T, F> {
        QueryFnMut(func, PhantomData)
    }
}

impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> ColMulti for QueryFnMut<T, F> {
    type T = T;
    #[inline(always)]
    fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
        self.0(a, b);
    }
}
impl<T, F> Splitter for QueryFnMut<T, F> {
    
    #[inline(always)]
    fn div(&mut self) -> (Self,Self) {
        unreachable!()
    }
    #[inline(always)]
    fn add(&mut self,_:Self, _: Self) {
        unreachable!()
    }
}

pub(super) struct QueryFn<T, F>(F, PhantomData<T>);
impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> QueryFn<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFn<T, F> {
        QueryFn(func, PhantomData)
    }
}
impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> ColMulti for QueryFn<T, F> {
    type T = T;

    #[inline(always)]
    fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
        self.0(a, b);
    }
}

impl<T, F: Clone> Splitter for QueryFn<T, F> {
    
    #[inline(always)]
    fn div(&mut self) -> (Self,Self) {
        (QueryFn(self.0.clone(), PhantomData),QueryFn(self.0.clone(), PhantomData))
    }
    #[inline(always)]
    fn add(&mut self,_:Self, _: Self) {}
}
