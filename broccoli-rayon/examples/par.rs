use broccoli::rect;
use broccoli_rayon::{build::RayonBuildPar, queries::colfind::RayonQueryPar};

fn main() {
    let mut inner1 = 0;
    let mut inner2 = 1;
    let mut inner3 = 2;

    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    let mut tree = broccoli::Tree::par_new(&mut aabbs);

    let mut res = tree.par_find_colliding_pairs_acc_closure(
        vec![],
        |_| vec![],
        |a, mut b| a.append(&mut b),
        |v, a, b| {
            v.push((*a.1, *b.1));
        },
    );

    res.sort();

    assert_eq!(res, vec!((1, 2), (2, 0)));
}
