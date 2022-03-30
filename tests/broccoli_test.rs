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

        let tree = broccoli::new(&mut bots);
        broccoli::queries::assert_tree_invariants(&tree);
        broccoli::queries::colfind::assert_query(&mut bots);
        
    }
}
