use broccoli::{tree::{bbox, rect}, queries::colfind::build::QueryDefault};
use broccoli::tree::node::BBox;
use broccoli::queries::colfind::CollidingPairsBuilder;
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    //Rect is stored directly in tree,
    //but inner is not.
    let mut aabbs = [
        bbox(rect(00, 10, 00, 10), &mut inner1),
        bbox(rect(15, 20, 15, 20), &mut inner2),
        bbox(rect(05, 15, 05, 15), &mut inner3),
    ];

    //This will change the order of the elements
    //in bboxes,but this is okay since we
    //populated it with mutable references.
    let mut tree = broccoli::tree::new_par(&mut aabbs);

    CollidingPairsBuilder::new(&mut tree, QueryDefault::new::<BBox<i32, &mut i32>>(|a, b| {
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    }))
    .build_par();

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 1);
    assert_eq!(inner3, 2);
}
