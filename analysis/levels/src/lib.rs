
use support::prelude::*;
use support::datanum;
use support::prelude::tree::BuildArgs;

use self::levelcounter::LevelCounter;
use self::leveltimer::LevelTimer;

mod levelcounter;
mod leveltimer;




use broccoli::queries::colfind::QueryArgs;



pub struct Res<X> {
    pub rebal: Vec<X>,
    pub query: Vec<X>,
}

pub fn bench(num:usize,start_grow:f64,end_grow:f64)->Vec<(f64,Res<f64>)>{
    grow_iter(start_grow, end_grow).map(|grow|{
        let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
        let res=gen(&mut all);
        (grow,res)
    }).collect()
}

pub fn theory(num:usize,start_grow:f64,end_grow:f64)->Vec<(f64,Res<usize>)>{
    grow_iter(start_grow, end_grow).map(|grow|{
        let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
        let res=gen_theory(&mut all);
        (grow,res)
    }).collect()
}


fn gen_theory<T:ColfindHandler>(bots:&mut [T])->Res<usize>{
    let (rebal, query) = datanum::datanum_test2(|maker| {
        
        maker.reset();

        let len = bots.len();
        let (mut tree, levelc) = Tree::from_build_args(
            bots,
            BuildArgs::new(len).with_splitter(LevelCounter::new(0, vec![])),
        );

        let c1 = levelc.into_levels().into_iter().map(|x| x ).collect();
        maker.reset();

        let levelc2 = tree.find_colliding_pairs_from_args(
            QueryArgs::new().with_splitter(LevelCounter::new(0, vec![])),
            |a, b| {
                T::handle(a.unpack_inner(),b.unpack_inner());
            },
        );

        let c2 = levelc2
            .into_levels()
            .into_iter()
            .map(|x| x )
            .collect();

        (c1, c2)
    });

    Res { rebal, query }
}


fn gen<T:ColfindHandler>(bots:&mut [T]) -> Res<f64> {
    
    let len = bots.len();
    let (mut tree, times1) = Tree::from_build_args(
        bots,
        BuildArgs::new(len).with_splitter(LevelTimer::new(0, vec![])),
    );

    let c1 = times1.into_levels().into_iter().map(|x| x as f64).collect();

    let times2 = tree.find_colliding_pairs_from_args(
        QueryArgs::new().with_splitter(LevelTimer::new(0, vec![])),
        |a, b| {
            T::handle(a.unpack_inner(), b.unpack_inner());
        },
    );

    let c2 = times2.into_levels().into_iter().map(|x| x as f64).collect();

    let t = Res {
        rebal: c1,
        query: c2,
    };

    assert_eq!(t.rebal.len(), t.query.len());
    assert_eq!(t.rebal.len(), t.query.len());
    t
}

