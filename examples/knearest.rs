use axgeom::vec2;
use broccoli::prelude::*;
use broccoli::tree::{bbox, rect};

fn distance_squared(a: isize, b: isize) -> isize {
    let a = (a - b).abs();
    a * a
}

fn main() {
    let mut inner1 = vec2(5, 5);
    let mut inner2 = vec2(3, 3);
    let mut inner3 = vec2(7, 7);

    let mut bots = [
        bbox(rect(0, 10, 0, 10), &mut inner1),
        bbox(rect(2, 4, 2, 4), &mut inner2),
        bbox(rect(6, 8, 6, 8), &mut inner3),
    ];

    let mut tree = broccoli::tree::new(&mut bots);

    let mut res = tree.k_nearest_mut_closure(
        vec2(30, 30),
        2,
        |point, a| Some(a.rect.distance_squared_to_point(point).unwrap_or(0)),
        |point, a| a.inner.distance_squared_to_point(point),
        |point, a| distance_squared(point.x, a),
        |point, a| distance_squared(point.y, a),
    );

    assert_eq!(res.len(), 2);
    assert_eq!(res.total_len(), 2);

    let foo: Vec<_> = res.iter().map(|a| *a[0].bot.inner).collect();

    tree.assert_tree_invariants();

    assert_eq!(foo, vec![vec2(7, 7), vec2(5, 5)])
}
