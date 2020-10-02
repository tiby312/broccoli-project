use axgeom;
use dinotree_alg::*;

#[test]
fn test1() {
    for &num_bots in [1000, 0, 1].iter() {
        let s = dists::spiral_iter([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .enumerate()
            .map(|(id, [x,y])| {
                bbox(
                    axgeom::Rect::from_point(axgeom::vec2(x as i64,y as i64), axgeom::vec2same(8 + id as i64)),
                    (),
                )
            })
            .collect();

        let mut tree=collections::DinoTreeRef::new(&mut bots);
        assert::Assert::find_intersections_mut(&mut tree);
    }
}
