use broccoli::{bbox_mut, rect, util::elem_offset};

fn main() {
    let mut inner = [0, 0, 0];
    let mut it = inner.iter_mut();

    // Rect is stored directly in tree,
    // but inner is not.
    let mut aabbs = [
        bbox_mut(rect(00, 10, 00, 10), it.next().unwrap()),
        bbox_mut(rect(15, 20, 15, 20), it.next().unwrap()),
        bbox_mut(rect(05, 15, 05, 15), it.next().unwrap()),
    ];

    // Construct tree by doing many swapping of elements
    let mut tree = broccoli::Tree::new(&mut aabbs);

    let mut pairs = vec![];

    // Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        pairs.push((
            std::ptr::from_ref(a.unpack()).addr(),
            std::ptr::from_ref(b.unpack()).addr(),
        ));
    });

    // Convert the raw addresses to indices.
    for (a, b) in &mut pairs {
        *a = elem_offset(&inner, *a).unwrap();
        *b = elem_offset(&inner, *b).unwrap();
    }

    // Get actual mutable references for each colliding pair.
    for &(index1, index2) in &pairs {
        let [a, b] = inner.get_disjoint_mut([index1, index2]).unwrap();
        *a += 1;
        *b += 1;
    }

    assert_eq!(inner[0], 1);
    assert_eq!(inner[1], 1);
    assert_eq!(inner[2], 2);
}
