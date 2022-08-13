use broccoli::tree::{node::Aabb, splitter::Splitter};

pub mod build;
pub mod query;




pub trait RayonPar<'a,T:Aabb>{
    fn par_new_ext<P:Splitter>(bots:&'a mut [T],num_level:usize,splitter:P,num_seq_fallback:usize)->(Self,P);
    fn par_new(bots:&'a mut [T])->Self;
    
}
