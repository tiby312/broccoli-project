use axgeom;

use broccoli::*;
use broccoli::prelude::*;
#[test]
fn test1() {
    for &num_bots in [0, 20, 100, 200].iter() {
        let s = dists::spiral_iter([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .enumerate()
            .map(|(_, [x, y])| {
                bbox(
                    axgeom::Rect::from_point(
                        axgeom::vec2(x as i64, y as i64),
                        axgeom::vec2same(8 as i64),
                    )
                    .into(),
                    (),
                )
            })
            .collect();

        let mut tree = broccoli::container::TreeRef::new(&mut bots);
        broccoli::analyze::assert::find_colliding_pairs_mut(&mut tree);
        assert!(broccoli::analyze::assert::tree_invariants(&*tree));
    }
}

#[test]
fn test2() {
    for &num_bots in [10, 0, 1].iter() {
        let s = dists::spiral_iter([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .enumerate()
            .map(|(id, [x, y])| {
                bbox(
                    axgeom::Rect::from_point(
                        axgeom::vec2(x as i64, y as i64),
                        axgeom::vec2same(8 + id as i64),
                    )
                    .into(),
                    (),
                )
            })
            .collect();

        let mut tree = broccoli::container::TreeRefInd::new(&mut bots, |a| a.rect);
        broccoli::analyze::assert::find_colliding_pairs_mut(&mut tree);
        broccoli::analyze::assert::find_colliding_pairs_mut(&mut tree);
        let mut p = tree.collect_colliding_pairs(|_a, _b| Some(()));
        let mut k = tree.collect_all(|_r, _a| Some(()));
        p.for_every_pair_mut(tree.get_elements_mut(), |_a, _b, _c| {});
        let _j: Vec<_> = k.get_mut(tree.get_elements_mut()).iter().collect();
        p.for_every_pair_mut(tree.get_elements_mut(), |_a, _b, _c| {});
    }
}


#[test]
fn test3() {
    for &num_bots in [0, 20, 100, 200].iter() {
        let s = dists::spiral_iter([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .map(|[x, y]| {
                bbox(
                    axgeom::Rect::from_point(
                        axgeom::vec2(x as i64, y as i64),
                        axgeom::vec2same(8 as i64),
                    )
                    .into(),
                    (),
                )
            })
            .collect();

        
        let mut tree = broccoli::container::TreeRefInd::new(&mut bots,|a|a.rect);

        let mut rects1=Vec::new();
        tree.find_colliding_pairs_mut(|a,b|rects1.push((a.rect,b.rect)));

        let rects2={
            use std::sync::Mutex;
            let mut rects=Mutex::new(Vec::new());
            let mut v=tree.collect_colliding_pairs_par(|a,b|Some(()));
            let mutex=&rects;
            v.for_every_pair_mut_par(tree.get_elements_mut(),|a,b,()|{
                let mut rects=mutex.lock().unwrap();
                rects.push((a.rect,b.rect))
            });
            rects.into_inner().unwrap()
        };
        let rects3={
            let mut rects=Vec::new();
            let mut v=tree.collect_colliding_pairs(|a,b|Some(()));
            v.for_every_pair_mut(tree.get_elements_mut(),|a,b,()|rects.push((a.rect,b.rect)));
            rects
        };

        assert_eq!(rects1.len(),rects2.len());
        assert_eq!(rects2.len(),rects3.len());

    }
}
