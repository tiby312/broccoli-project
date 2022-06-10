use support::prelude::*;



#[inline(never)]
pub fn bench(
    max: usize,
    max_height:usize,
    grow: f64,
) -> Vec<(usize, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max_height)
        .map(move |height| {
            let f=new_bench_record(&mut all, height);
            (height,f)
        }).collect()
}

#[inline(never)]
pub fn theory(
    man:&mut DnumManager,
    max: usize,
    max_height:usize,
    grow: f64,
) -> Vec<(usize,usize)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max_height)
        .map(move |height| {
            let f=new_theory_record(man,&mut all, height);
            (height,f)
        }).collect()
}


pub struct Res{
    pub optimal_height:usize,
    pub heur_height:usize
}


#[inline(never)]
pub fn optimal(num:usize,grow:f64)->Vec<(usize,Res)>{
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

    (0..num).step_by(1000).map(move |n|{
        let bots=&mut all[0..n];

        let optimal_height=(0..20).map(|height|{
            (height,new_bench_record(bots,height))
        }).min_by(|(_,a),(_,b)|a.partial_cmp(b).unwrap()).unwrap().0;


        let b=BuildArgs::new(n);
        let heur_height=b.num_level;

        (n,Res{
            optimal_height,
            heur_height
        })
    }).collect()
}


fn new_theory_record<T:ColfindHandler>(man:&mut DnumManager,bots:&mut [T],height:usize) -> usize{
    
    man.time(||{
        let len = bots.len();
        let (mut tree, _) =
            Tree::from_build_args(bots, BuildArgs::new(len).with_num_level(height));

        assert_eq!(tree.num_levels(), height);

        tree.find_colliding_pairs(T::handle);
    })
}

fn new_bench_record<T:ColfindHandler>(bots:&mut [T],height:usize) -> f64{
    let mut bencher=Bencher;

    bencher.time(||{
        let len = bots.len();
        let (mut tree, _) =
            Tree::from_build_args(bots, BuildArgs::new(len).with_num_level(height));

        assert_eq!(tree.num_levels(), height);

        tree.find_colliding_pairs(T::handle);
    })
}


