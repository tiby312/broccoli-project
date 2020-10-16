use axgeom;
use broccoli::prelude::*;

#[test]
fn test1() {
    for &num_bots in [30, 0, 1].iter() {
        let s = dists::spiral_iter([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .enumerate()
            .map(|(id, [x,y])| {
                bbox(
                    axgeom::Rect::from_point(axgeom::vec2(x as i64,y as i64), axgeom::vec2same(8 + id as i64)).into(),
                    (),
                )
            })
            .collect();

        let mut tree=broccoli::collections::TreeRef::new(&mut bots);
        broccoli::assert::find_colliding_pairs_mut(&mut tree);
    }
}
