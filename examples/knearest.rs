use broccoli::axgeom::vec2;
use broccoli::tree::rect;

fn distance_squared(a: isize, b: isize) -> isize {
    let a = (a - b).abs();
    a * a
}

fn main() {
    let mut inner1 = vec2(5, 5);
    let mut inner2 = vec2(3, 3);
    let mut inner3 = vec2(7, 7);

    let mut bots = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(02, 04, 02, 04), &mut inner2),
        (rect(06, 08, 06, 08), &mut inner3),
    ];

    let mut tree = broccoli::Tree2::new(&mut bots);

    let mut res = tree.find_knearest_closure(
        vec2(30, 30),
        2,
        |point, a| Some(a.0.distance_squared_to_point(point).unwrap_or(0)),
        |point, a| a.1.distance_squared_to_point(point),
        |point, a| distance_squared(point.x, a),
        |point, a| distance_squared(point.y, a),
    );

    assert_eq!(res.len(), 2);
    assert_eq!(res.total_len(), 2);

    let foo: Vec<_> = res.iter().map(|a| *a[0].bot.1).collect();

    tree.assert_tree_invariants();

    assert_eq!(foo, vec![vec2(7, 7), vec2(5, 5)])
}
