use support::prelude::*;



pub fn num_intersection(_emp:&mut Html,man:&mut DnumManager)->std::fmt::Result{
    unimplemented!();
}
pub fn handle_grow(_emp:&mut Html)->std::fmt::Result{
    unimplemented!();
}
pub fn handle_visualize(_emp:&mut Html)->std::fmt::Result{
    unimplemented!();
}


#[inline(never)]
pub fn num_intersection_inner(max: usize, grow: f64) -> Vec<(usize, usize)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(10)
        .skip(1)
        .map(|a| {
            let bots = &mut all[0..a];

            let mut tree = Tree::new(bots);
            let mut num_intersection = 0;
            tree.find_colliding_pairs(|_a, _b| {
                num_intersection += 1;
            });
            (a, num_intersection)
        })
        .collect()
}

#[inline(never)]
pub fn handle_grow_inner(num: usize, min_grow: f64, max_grow: f64) -> Vec<(f64, usize)> {
    grow_iter(min_grow, max_grow)
        .map(|grow| {
            let mut bots: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

            let mut tree = Tree::new(&mut bots);

            let mut num_intersection = 0;
            tree.find_colliding_pairs(|_a, _b| {
                num_intersection += 1;
            });

            (grow, num_intersection)
        })
        .collect()
}

#[inline(never)]
pub fn handle_visualize_inner(grow: f64, num: usize) -> Vec<[f32; 2]> {
    dist::dist(grow)
        .map(|x| Dummy(x, 0u32))
        .take(num)
        .map(|a| [a.0.x.start, a.0.y.start])
        .collect()
}
