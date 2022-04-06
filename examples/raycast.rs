use axgeom::vec2;
use broccoli::prelude::*;
use broccoli::tree::{bbox, rect};

fn main() {
    let mut inner1 = 4;
    let mut inner2 = 1;
    let mut inner3 = 2;

    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), &mut inner1),
        bbox(rect(15, 20, 15, 20), &mut inner2),
        bbox(rect(5, 15, 5, 15), &mut inner3),
    ];

    let mut tree = broccoli::tree::new(&mut aabbs);

    let ray = axgeom::Ray {
        point: vec2(-10, 1),
        dir: vec2(1, 0),
    };

    let res = tree.raycast_mut_closure(
        ray,
        | _, _| None,
        |ray, a| ray.cast_to_rect(&a.rect),
        |ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
        |ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
    );

    assert_eq!(*res.unwrap().elems[0].inner, 4);

    tree.assert_tree_invariants();
}
