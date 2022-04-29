use broccoli::prelude::*;
use broccoli::tree::{aabb_pin::AabbPin, rect};

fn main() {
    let inner1 = 4;
    let inner2 = 5;
    let inner3 = 6;

    let mut bots = [
        (rect(00, 10, 00, 10), &inner1),
        (rect(15, 20, 15, 20), &inner2),
        (rect(05, 15, 05, 15), &inner3),
    ];

    let mut tree = broccoli::tree::new(&mut bots);

    let mut rect_collisions = Vec::new();
    tree.for_all_intersect_rect_mut(AabbPin::new(&mut rect(-5, 1, -5, 1)), |_, a| {
        rect_collisions.push(a);
    });

    assert_eq!(rect_collisions.len(), 1);
    assert_eq!(rect_collisions[0].0, rect(0, 10, 0, 10));
    assert_eq!(*rect_collisions[0].1, 4);

    tree.assert_tree_invariants();
}
