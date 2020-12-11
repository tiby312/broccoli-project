### Overview

Broccoli is a broadphase collision detection library. 

The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).

Checkout the [github repo](https://github.com/tiby312/broccoli) and on [crates.io](https://crates.io/crates/broccoli).

### Inner projects

The `broccoli_demo` inner project is meant to show case the use of these algorithms. 
The report inner project generates benches used in the [broccoli book](https://tiby312.github.io/broccoli_report).

### Screenshot

Screen capture from the inner `broccoli_demo` project.

<img src="./assets/screenshot.gif" alt="screenshot">

### Example

```rust
use broccoli::{bbox, prelude::*, rect};
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

    //Find all colliding aabbs.
    tree.find_colliding_pairs_mut(|a, b| {
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 0);
    assert_eq!(inner3, 1);
}

```
