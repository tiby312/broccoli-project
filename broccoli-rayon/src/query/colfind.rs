use broccoli::{tree::{node::Aabb, splitter::Splitter, aabb_pin::AabbPin}, Tree};







pub trait RayonQueryPar<'a,T:Aabb>{
    
    fn par_find_colliding_pairs_ext<S: Splitter, F>(
        &mut self,
        num_switch_seq:usize,
        func:F
    )->S
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        S: Send,
        T: Send,
        T::Num: Send;
    
    
}

impl<'a,T:Aabb> RayonQueryPar<'a,T> for Tree<'a,T>{
    fn par_find_colliding_pairs_ext<S: Splitter, F>(
        &mut self,
        num_switch_seq:usize,
        func:F
    )->S
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        S: Send,
        T: Send,
        T::Num: Send
    {
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
            prevec: PreVec::new(),
        };
        args.par_query(self.vistr_mut(), &mut f)
        
    }

}


pub struct ParQuery{
    pub num_seq_fallback:usize
}
impl Default for ParQuery{
    fn default()->Self{
        ParQuery{
            num_seq_fallback:SEQ_FALLBACK_DEFAULT
        }
    }
}



impl PayQuery{
    pub fn par_query<P,T: Aabb, SO>(mut self, splitter:P,vistr: VistrMutPin<Node<T>>, handler: &mut SO) -> P
    where
        P: Splitter,
        SO: NodeHandler<T>,
        T: Send,
        T::Num: Send,
        SO: Splitter + Send,
        P: Send,
    {
        let vv = CollVis::new(vistr);
        recurse_par(vv, &mut splitter, handler, self.num_seq_fallback);
        splitter
    }


    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_from_args<S: Splitter, F>(
        &mut self,
        args: QueryArgs<S>,
        func: F,
    ) -> S
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        S: Send,
        T: Send,
        T::Num: Send,
    {
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
            prevec: PreVec::new(),
        };
        args.par_query(self.vistr_mut(), &mut f)
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_acc<Acc: Splitter, F>(&mut self, acc: Acc, func: F) -> Acc
    where
        F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        Acc: Splitter + Send,
        T: Send,
        T::Num: Send,
    {
        let floop = Floop { acc, func };
        let mut f = AccNodeHandler {
            acc: floop,
            prevec: PreVec::new(),
        };
        QueryArgs::new().par_query(self.vistr_mut(), &mut f);
        f.acc.acc
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
        &mut self,
        acc: Acc,
        div: A,
        add: B,
        func: F,
    ) -> Acc
    where
        A: FnMut(&mut Acc) -> Acc + Clone + Send,
        B: FnMut(&mut Acc, Acc) + Clone + Send,
        F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        Acc: Send,
        T: Send,
        T::Num: Send,
    {
        let floop = FloopClosure {
            acc,
            div,
            add,
            func,
        };

        let mut f = AccNodeHandler {
            acc: floop,
            prevec: PreVec::new(),
        };
        QueryArgs::new().par_query(self.vistr_mut(), &mut f);
        f.acc.acc
    }


}

fn recurse_par<T: Aabb, P: Splitter, SO: NodeHandler<T>>(
    vistr: CollVis<T>,
    splitter: &mut P,
    handler: &mut SO,
    num_seq_fallback: usize,
) where
    T: Send,
    T::Num: Send,
    SO: Splitter + Send,
    P: Send,
{
    if vistr.num_elem() <= num_seq_fallback {
        recurse_seq(vistr, splitter, handler);
    } else {
        let (n, rest) = vistr.collide_and_next(handler);
        if let Some([left, right]) = rest {
            let mut splitter2 = splitter.div();
            let mut h2 = handler.div();

            rayon::join(
                || {
                    n.finish(handler);
                    recurse_par(left, splitter, handler, num_seq_fallback)
                },
                || recurse_par(right, &mut splitter2, &mut h2, num_seq_fallback),
            );
            handler.add(h2);
            splitter.add(splitter2);
        } else {
            n.finish(handler);
        }
    }
}
