use broccoli::prelude::*;
fn main() {
    let mut inner1=0;
    let mut inner2=0;
    let mut inner3=0;

    //Using a semi-direct layout for best performance.
    //rect is stored directly in tree, but inner is not.
    let mut aabbs = [
        broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), &mut inner1),
        broccoli::bbox(broccoli::rect(15, 20, 15, 20), &mut inner2),
        broccoli::bbox(broccoli::rect(5, 15, 5, 15), &mut inner3),
    ];

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::new(&mut aabbs);

    //Find all colliding aabbs.
    tree.find_colliding_pairs_mut(|a, b| {
        **a += 1;
        **b += 1;
    });

    assert_eq!(*aabbs[0].inner, 1);
    assert_eq!(*aabbs[1].inner, 0);
    assert_eq!(*aabbs[2].inner, 1);
}
