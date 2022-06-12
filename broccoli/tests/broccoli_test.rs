use axgeom;
use broccoli::Assert;
#[test]
fn test1() {
    for &i in [2.0, 4.0, 12.0].iter() {
        for &num_bots in [0, 20, 40, 10000].iter() {
            let s = dists::spiral_iter([400.0, 400.0], i, 1.0);

            let mut data: Vec<_> = (0..num_bots).collect();

            let mut bots: Vec<_> = s
                .take(num_bots)
                .zip(data.iter_mut())
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

            let tree = broccoli::Tree::new(&mut bots);
            tree.assert_tree_invariants();
            let data = tree.get_tree_data();
            let mut tree = broccoli::Tree::from_tree_data(&mut bots, &data);
            tree.assert_tree_invariants();
            tree.find_colliding_pairs(|a, b| {
                let a = a.unpack_inner();
                let b = b.unpack_inner();
                **a ^= 1;
                **b ^= 1;
            });

            Assert::new(&mut bots).assert_query();
        }
    }
}