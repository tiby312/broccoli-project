use broccoli_rayon::queries::colfind::RayonQueryPar;
#[test]
fn test1() {
    for &i in [2.0, 4.0, 12.0].iter() {
        for &num_bots in [0, 20, 40, 10000].iter() {
            let s = dists::spiral_iter([400.0, 400.0], i, 1.0);

            let data: Vec<_> = (0..num_bots).collect();

            let mut bots: Vec<_> = s
                .take(num_bots)
                .zip(data.into_iter())
                .map(|([x, y], dd)| {
                    (
                        axgeom::Rect::from_point(
                            axgeom::vec2(x as i64, y as i64),
                            axgeom::vec2same(8 as i64),
                        )
                        .into(),
                        dd,
                    )
                })
                .collect();

            let mut bots: Vec<_> = bots.iter_mut().collect();

            let mut tree = broccoli::Tree::new(&mut bots);

            let mut vs = vec![];
            tree.find_colliding_pairs(|a, b| {
                vs.push((a.1, b.1));
            });

            let mut vs2 = tree.par_find_colliding_pairs_acc_closure(
                vec![],
                |_| vec![],
                |a, mut b| a.append(&mut b),
                |v, a, b| {
                    v.push((a.1, b.1));
                },
            );

            vs.sort();
            vs2.sort();
            assert_eq!(vs, vs2);
        }
    }
}
