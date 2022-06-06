use support::prelude::*;

trait TestTrait: Copy + Send + Sync {
    fn handle(&mut self,other:&mut Self);
}


impl<const K:usize> TestTrait for [u8;K] {
    fn handle(&mut self,other:&mut Self){
        self[0]^=1;
        other[0]^=1;
    }
}


const MAX:usize=30_000;



fn test_direct<T:TestTrait>(grow:f64,val:T)->Vec<(usize,f64,f64)>{
    let mut all = dist::create_dist_manyswap(MAX, grow, val);
    test_one_kind(&mut all,|a,b|{
        a.unpack_inner().handle(b.unpack_inner())
    })
}

fn test_indirect<T:TestTrait>(grow:f64,val:T)->Vec<(usize,f64,f64)>{
    let mut all = dist::create_dist_manyswap(MAX, grow, val);
    let mut ind:Vec<_>=all.iter_mut().collect();
    test_one_kind(&mut ind,|a,b|{
        a.unpack_inner().handle(b.unpack_inner())
    })
}

fn test_default<T:TestTrait>(grow:f64,val:T)->Vec<(usize,f64,f64)>{
        
    let mut all = dist::create_dist_manyswap(MAX, grow, val);
    let mut ind:Vec<_>=all.iter_mut().map(|x|(x.0.0,&mut x.0.1)).collect();
    test_one_kind(&mut ind,|a,b|{
        a.unpack_inner().handle(b.unpack_inner())
    })
}


fn test_one_kind<T:Aabb+ManySwap>(all:&mut [T],mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>))->Vec<(usize,f64,f64)>{
    let mut plots=vec!();
    for a in n_iter(0,MAX){
        let bots=&mut all[0..a];

        let (mut tree, construct_time) = bench_closure_ret(|| broccoli::Tree::new(bots));

        let (_tree, query_time) = bench_closure_ret(|| {
            tree.find_colliding_pairs(|a, b| {
                func(a, b);
            });
            tree
        });    
    
        plots.push((a,construct_time,query_time));
    }
    plots
}


pub enum Layout{
    Direct,
    Indirect,
    Default
}

#[inline(never)]
pub fn bench(typ:Layout,grow:f64,size:usize)->Vec<(usize,f64,f64)>{
    match typ{
        Layout::Direct=>{
            match size{
                8=>test_direct(grow,[0u8;8]),
                128=>test_direct(grow,[0u8;128]),
                256=>test_direct(grow,[0u8;256]),
                _=>panic!("invalid size")
            }
            
        },
        Layout::Indirect=>{
            match size{
                8=>test_indirect(grow,[0u8;8]),
                128=>test_indirect(grow,[0u8;128]),
                256=>test_indirect(grow,[0u8;256]),
                _=>panic!("invalid size")
            }
            
        },
        Layout::Default=>{
            match size{
                8=>test_default(grow,[0u8;8]),
                128=>test_default(grow,[0u8;128]),
                256=>test_default(grow,[0u8;256]),
                _=>panic!("invalid size")
            }
        }
    }
}

