use broccoli::prelude::*;
use broccoli::collections::TreeRefInd;
fn main() {
    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), 0),
        bbox(rect(15, 20, 15, 20), 0),
        bbox(rect(5, 15, 5, 15), 0),
    ];

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = TreeRefInd::new(&mut aabbs,|a|a.rect);

    //Find all colliding aabbs.
    tree.find_colliding_pairs_mut(|a, b| {
        a.inner += 1;
        b.inner += 1;
    });

    assert_eq!(aabbs[0].inner, 1);
    assert_eq!(aabbs[1].inner, 0);
    assert_eq!(aabbs[2].inner, 1);
}
