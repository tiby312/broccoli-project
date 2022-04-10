use broccoli::prelude::*;
use broccoli::tree::{bbox, rect};
fn main() {
    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), 0),
        bbox(rect(15, 20, 15, 20), 0),
        bbox(rect(5, 15, 5, 15), 0),
    ];

    //Create a layer of direction.
    let mut ref_aabbs = aabbs.iter_mut().collect::<Vec<_>>();

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::tree::new(&mut ref_aabbs);

    //Find all colliding aabbs.
    tree.colliding_pairs(|a, b| {
        *a.unpack_inner() += 1;
        *b.unpack_inner() += 1;
    });

    assert_eq!(aabbs[0].inner, 1);
    assert_eq!(aabbs[1].inner, 0);
    assert_eq!(aabbs[2].inner, 1);
}
