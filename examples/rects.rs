use axgeom::vec2;
use broccoli::{bbox, prelude::*, rect};

fn main() {
    let mut inner1 = vec2(5, 5);
    let mut inner2 = vec2(3, 3);
    let mut inner3 = vec2(7, 7);

    let mut bots = [
        bbox(rect(0isize, 10, 0, 10), &mut inner1),
        bbox(rect(15, 20, 15, 20), &mut inner2),
        bbox(rect(5, 15, 5, 15), &mut inner3),
    ];

    let mut tree = broccoli::collections::TreeRef::new(&mut bots);

    //Here we query for read-only references so we can pull
    //them out of the closure.
    let mut rect_collisions = Vec::new();
    tree.for_all_intersect_rect(&rect(-5, 1, -5, 1), |a| {
        rect_collisions.push(a);
    });

    assert_eq!(rect_collisions.len(), 1);
    assert_eq!(*rect_collisions[0].get(), rect(0, 10, 0, 10));
}
