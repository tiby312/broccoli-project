use support::prelude::*;

#[derive(Debug)]
pub struct Res {
    pub bench: f64,
    pub collect: f64,
}


#[inline(never)]
pub fn bench(max:usize,grow:f64,num_iter:usize)-> Vec<(usize,Res)>{
    assert!(num_iter>=1);
    let mut bencher=Bencher;
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .map(|n| {
            let bots=&mut all[0..n];

            let control=bencher.time(||{
                let mut t=Tree::new(bots);
                for _ in 0..num_iter{
                    t.find_colliding_pairs(Dummy::<f32, u32>::handle);
                }
            });

            let test=bencher.time(||{
                let mut tree=Tree::new(bots);
                let mut tree = broccoli::ext::cacheable_pairs::IndTree(&mut tree);
                let mut cacher = broccoli::ext::cacheable_pairs::CacheSession::new(&mut tree);
                let mut pairs = cacher.cache_colliding_pairs(|a,b|{
                    *a ^= 1;
                    *b ^= 1;
                    Some(())
                });

                for _ in 1..num_iter {
                    pairs.handle(&mut cacher, |a, b, ()| {
                        *a ^= 1;
                        *b ^= 1;
                    });
                }
                
            });

            (n,Res{
                bench:control,
                collect:test
            })
        }).collect()
}

