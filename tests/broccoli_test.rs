use axgeom;
use broccoli::*;
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

        let mut tree = broccoli::new(&mut bots);
        broccoli::assert::assert_query(&mut tree);
        broccoli::assert::assert_tree_invariants(&tree);
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

        let mut base = broccoli::container::TreeIndBase::new(&mut bots, |a| a.rect);
        let mut tree = base.build();
        broccoli::assert::assert_query(&mut *tree);
        let mut p = tree.collect_colliding_pairs(|_a, _b| Some(()));
        let mut k = tree.collect_all(|_r, _a| Some(()));
        p.for_every_pair_mut(tree.get_inner_elements_mut(), |_a, _b, _c| {});
        let _j: Vec<_> = k.get_mut(tree.get_inner_elements_mut()).iter().collect();
        p.for_every_pair_mut(tree.get_inner_elements_mut(), |_a, _b, _c| {});
    }
}

#[test]
fn test3() {
    for &num_bots in [0, 20, 100, 200, 1000].iter() {
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

        let mut base = broccoli::container::TreeIndBase::new(&mut bots, |a| a.rect);
        let mut tree = base.build();

        let mut rects1 = Vec::new();
        tree.find_colliding_pairs_mut(|a, b| rects1.push((a.rect, b.rect)));

        let rects2 = {
            use std::sync::Mutex;
            let rects = Mutex::new(Vec::new());
            let mut v = tree.collect_colliding_pairs_par(RayonJoin, |_, _| Some(()));
            dbg!(v.get(tree.get_inner_elements_mut()).len());

            let mutex = &rects;
            v.for_every_pair_mut_par(RayonJoin, tree.get_inner_elements_mut(), |a, b, ()| {
                let mut rects = mutex.lock().unwrap();
                rects.push((a.rect, b.rect))
            });
            rects.into_inner().unwrap()
        };
        let rects3 = {
            let mut rects = Vec::new();
            let mut v = tree.collect_colliding_pairs(|_, _| Some(()));
            v.for_every_pair_mut(tree.get_inner_elements_mut(), |a, b, ()| {
                rects.push((a.rect, b.rect))
            });
            rects
        };

        //TODO assert all the same.
        assert_eq!(rects1.len(), rects2.len());
        assert_eq!(rects2.len(), rects3.len());
    }
}
