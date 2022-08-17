use broccoli::rect;

fn main() {
    let mut aabbs = [
        (rect(00, 10, 00, 10), 0),
        (rect(15, 20, 15, 20), 0),
        (rect(05, 15, 05, 15), 0),
    ];

    //Create a layer of direction.
    let mut ref_aabbs = aabbs.iter_mut().collect::<Vec<_>>();

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::Tree::new(&mut ref_aabbs);

    //Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        *a.unpack_inner() += 1;
        *b.unpack_inner() += 1;
    });

    assert_eq!(aabbs[0].1, 1);
    assert_eq!(aabbs[1].1, 1);
    assert_eq!(aabbs[2].1, 2);
}
