//! Provides 2d broadphase collision detection.

pub mod oned;

use super::tools;
use super::*;
pub mod build;
use build::*;
pub mod handler;

mod assert {
    use super::*;
    impl<'a, T: Aabb> Assert<'a, T> {
        ///Panics if a disconnect is detected between all colfind methods.
        pub fn assert_query(&mut self) {
            let bots = &mut self.inner;
            #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
            pub struct CollisionPtr {
                inner: Vec<(usize, usize)>,
            }

            impl CollisionPtr {
                fn new() -> Self {
                    CollisionPtr { inner: vec![] }
                }
                fn add_pair(&mut self, a: usize, b: usize) {
                    let (a, b) = if a < b { (a, b) } else { (b, a) };

                    self.inner.push((a, b));
                }
                pub fn finish(&mut self) {
                    self.inner.sort_unstable();
                }
            }

            let mut bots: Vec<_> = bots
                .iter_mut()
                .enumerate()
                .map(|(i, x)| ManySwappable((*x.get(), i)))
                .collect();
            let bots = bots.as_mut_slice();

            let naive_res = {
                let mut cc = CollisionPtr::new();
                Naive::new(bots).find_colliding_pairs(|a, b| {
                    cc.add_pair(a.0 .1, b.0 .1);
                });
                cc.finish();
                cc
            };

            let tree_res = {
                let mut cc = CollisionPtr::new();

                Tree::new(bots).find_colliding_pairs(|a, b| {
                    cc.add_pair(a.0 .1, b.0 .1);
                });
                cc.finish();
                cc
            };

            // let notsort_res = {
            //     let mut cc = CollisionPtr::new();

            //     NotSortedTree::new(bots).find_colliding_pairs(|a, b| {
            //         cc.add_pair(a.0 .1, b.0 .1);
            //     });
            //     cc.finish();
            //     cc
            // };

            // let sweep_res = {
            //     let mut cc = CollisionPtr::new();
            //     SweepAndPrune::new(bots).find_colliding_pairs(|a, b| {
            //         cc.add_pair(a.0 .1, b.0 .1);
            //     });
            //     cc.finish();
            //     cc
            // };

            //assert_eq!(naive_res.inner.len(), sweep_res.inner.len());
            assert_eq!(naive_res.inner.len(), tree_res.inner.len());
            //assert_eq!(naive_res.inner.len(), notsort_res.inner.len());

            assert_eq!(naive_res, tree_res);
            //assert_eq!(naive_res, sweep_res);
            //assert_eq!(naive_res, notsort_res);
        }
    }

    impl<'a, T: Aabb> Naive<'a, T> {
        pub fn find_colliding_pairs(
            &mut self,
            mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        ) {
            queries::for_every_pair(self.inner.borrow_mut(), move |a, b| {
                if a.get().intersects_rect(b.get()) {
                    func(a, b);
                }
            });
        }
    }
}
