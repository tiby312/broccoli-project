use support::prelude::*;

pub fn num_intersection(emp: &mut Html) -> std::fmt::Result {
    let n = 10_000;
    let grow = 2.0;
    let description = formatdoc! {r#"
            Num of comparison for
            `abspiral(n,{grow})`
        "#};

    let res = num_intersection_inner(n, 2.0);

    let p = plots!(
        res.iter().cloned_plot().scatter(""),
        (0..10_000).map(|x|[x,x]).cloned_plot().line("n"),
        poloto::build::markers([], [0])
    );

    emp.write_graph(
        Some("spiral"),
        &format!("num_intersection{}", n),
        "num elements",
        "num comparison",
        p,
        &description,
    )
}
pub fn handle_grow(emp: &mut Html) -> std::fmt::Result {
    let n = 10_000;
    let description = formatdoc! {r#"
            Num of comparison for
            `abspiral({n},x)`
        "#};

    let res = handle_grow_inner(n, 0.5, 2.0);

    let p = plots!(
        res.iter().cloned_plot().scatter(""),
        [(0.5,10_000*2),(2.0,10_000*2)].iter().cloned_plot().line("n*2"),
        poloto::build::markers([], [0])
    );

    emp.write_graph(
        Some("spiral"),
        &format!("num_intersection{}", n),
        "grow",
        "num comparison",
        p,
        &description,
    )
}
pub fn handle_visualize(emp: &mut Html) -> std::fmt::Result {
    let n = 500;
    let description = formatdoc! {r#"
            visual of
            `abspiral({n},x)`
        "#};

    let res = handle_visualize_inner(2.0, n);

    let p = plots!(
        res.iter().cloned_plot().scatter(""),
        poloto::build::markers([], [0.0])
    );

    let mut opt = poloto::render::render_opt_builder();
    opt.preserve_aspect();
    emp.write_graph_ext(
        opt.build(),
        Some("spiral"),
        &format!("spiral{}", n),
        "x",
        "y",
        p,
        &description,
    )
}

#[inline(never)]
pub fn num_intersection_inner(max: usize, grow: f64) -> Vec<(i128, i128)> {
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
            (a as i128, num_intersection as i128)
        })
        .collect()
}

#[inline(never)]
pub fn handle_grow_inner(num: usize, min_grow: f64, max_grow: f64) -> Vec<(f64, i128)> {
    grow_iter(min_grow, max_grow)
        .map(|grow| {
            let mut bots: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();

            let mut tree = Tree::new(&mut bots);

            let mut num_intersection = 0;
            tree.find_colliding_pairs(|_a, _b| {
                num_intersection += 1;
            });

            (grow, num_intersection as i128)
        })
        .collect()
}

#[inline(never)]
pub fn handle_visualize_inner(grow: f64, num: usize) -> Vec<[f64; 2]> {
    dist::dist(grow)
        .map(|x| Dummy(x, 0u32))
        .take(num)
        .map(|a| [a.0.x.start as f64, a.0.y.start as f64])
        .collect()
}
