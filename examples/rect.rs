use broccoli::prelude::*;
use broccoli::tree::{aabb_pin::AabbPin, bbox, rect};

fn main() {
    let inner1 = 4;
    let inner2 = 5;
    let inner3 = 6;

    let mut bots = [
        bbox(rect(0isize, 10, 0, 10), &inner1),
        bbox(rect(15, 20, 15, 20), &inner2),
        bbox(rect(5, 15, 5, 15), &inner3),
    ];

    let mut tree = broccoli::tree::new(&mut bots);

    let mut rect_collisions = Vec::new();
    tree.for_all_intersect_rect_mut(AabbPin::new(&mut rect(-5, 1, -5, 1)), |_, a| {
        rect_collisions.push(a);
    });

    assert_eq!(rect_collisions.len(), 1);
    assert_eq!(rect_collisions[0].rect, rect(0, 10, 0, 10));
    assert_eq!(*rect_collisions[0].inner, 4);

    tree.assert_tree_invariants();
}
