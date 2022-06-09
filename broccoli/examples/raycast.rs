use broccoli::axgeom;
use broccoli::axgeom::vec2;

use broccoli::tree::rect;

fn main() {
    let mut inner1 = 4;
    let mut inner2 = 1;
    let mut inner3 = 2;

    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    let mut tree = broccoli::Tree::new(&mut aabbs);

    let ray = axgeom::Ray {
        point: vec2(-10, 1),
        dir: vec2(1, 0),
    };

    let res = tree.cast_ray_closure(
        ray,
        |_, _| None,
        |ray, a| ray.cast_to_rect(&a.0),
        |ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
        |ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
    );

    assert_eq!(*res.unwrap().elems[0].1, 4);

    tree.assert_tree_invariants();
}
