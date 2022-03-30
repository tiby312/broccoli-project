use broccoli::{bbox, rect};
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    //Rect is stored directly in tree,
    //but inner is not.
    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), &mut inner1),
        bbox(rect(15, 20, 15, 20), &mut inner2),
        bbox(rect(5, 15, 5, 15), &mut inner3),
    ];

    //This will change the order of the elements
    //in bboxes,but this is okay since we
    //populated it with mutable references.
    let mut tree = broccoli::new(&mut aabbs);

    let mut prevec = broccoli::util::PreVec::new();

    //Find all colliding aabbs.
    tree.colliding_pairs()
        .recurse_seq(&mut prevec, &mut |a, b| {
            **a.unpack_inner() += 1;
            **b.unpack_inner() += 1;
        });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 0);
    assert_eq!(inner3, 1);
}
