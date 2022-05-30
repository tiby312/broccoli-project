### Broccoli

[![Crates.io](https://img.shields.io/crates/v/broccoli)](https://crates.io/crates/broccoli)
[![docs.rs](https://docs.rs/broccoli/badge.svg)](https://docs.rs/broccoli)
[![Crates.io](https://img.shields.io/crates/d/broccoli)](https://crates.io/crates/broccoli)

Broccoli is a broad-phase collision detection library. 

The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).

Checkout it out on [github](https://github.com/tiby312/broccoli) and on [crates.io](https://crates.io/crates/broccoli). Documentation at [docs.rs](https://docs.rs/broccoli). 
### Screenshot

Screen capture from the inner `demo` project.

<img src="./assets/screenshot.gif" alt="screenshot">


### Example

```rust
use broccoli::tree::rect;
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    // Rect is stored directly in tree,
    // but inner is not.
    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    // Construct tree by doing many swapping of elements
    let mut tree = broccoli::Tree::new(&mut aabbs);

    // Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        // We aren't given &mut T reference, but instead of AabbPin<&mut T>.
        // We call unpack_inner() to extract the portion that we are allowed to mutate.
        // (We are not allowed to change the bounding box while in the tree)
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 1);
    assert_eq!(inner3, 2);
}
```


### Optimisation

I've focused mainly on making finding colliding pairs as fast as possible primarily in
distributions where there are a lot of overlapping aabbs.

Quick rundown of what i've spent effort on and a rough estimate of performance
cost of each algorithm in general. 

| Algorithm        | Cost | Effort spent  |
| ---------------- | ---- | ------------- |
| Construction     |   7  |        10     |
| Colliding Pairs  |   8  |        10     |
| Collide With     |   3  |         2     |
| knearest         |   1  |         2     |
| raycast          |   1  |         2     |
| rect             |   1  |         2     |
| nbody            |  10  |         1     |

Numbers are out of 10 and are just rough made up numbers. For more in-depth analysis, see the [broccoli book](https://tiby312.github.io/broccoli_report).


### Name

If you shorten "broad-phase collision" to "broad colli" and say it fast, it sounds like broccoli.
Broccoli are also basically small trees and broccoli uses a tree data structure.