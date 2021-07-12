use crate::inner_prelude::*;

type LTree = compt::dfs_order::CompleteTreeContainer<usize, compt::dfs_order::PreOrder>;

struct TheoryRes {
    query: LTree,
}
impl TheoryRes {
    fn new(num_bots: usize, grow: f64) -> TheoryRes {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f64)).collect();

        let query = datanum::datanum_test2(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));

            let mut tree = TreeBuilder::new(&mut bots).build_seq();

            maker.reset();

            let levelc2 = tree.new_colfind_builder().query_with_splitter_seq(
                |a, b| {
                    a.unpack_inner().x += 1.0;
                    b.unpack_inner().y += 1.0;
                },
                LevelCounter::new(),
            );

            levelc2.into_tree()
        });

        TheoryRes { query }
    }
}

pub fn handle2(fb: &mut FigureBuilder, prefix: &str, grow: f64, num_bots: usize) {
    {
        let res = TheoryRes::new(num_bots, grow);

        let mut splot = poloto::plot_with_html(
            move_format!(
                "Complexity of query evenness with abspiral({},{})",
                num_bots,
                grow
            ),
            "DFS inorder iteration",
            "Number of comparisons",
            REPORT_THEME
        );

        splot.histogram(
            "",
            res.query
                .vistr()
                .dfs_inorder_iter()
                .enumerate()
                .map(|(i, element)| [i as f64, *element as f64])
                .twice_iter(),
        );

        fb.finish_plot(splot, move_format!("query_evenness_theory_{}", prefix));
    }

    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f64)).collect();

    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

    let tree = broccoli::new(&mut bots);

    let mut splot = poloto::plot_with_html(
        move_format!("Num per node with abspiral({},{})", num_bots, grow),
        "DFS inorder iteration",
        "Number of comparisons",
        REPORT_THEME
    );

    use broccoli::compt::Visitor;
    splot.histogram(
        "",
        tree.vistr()
            .dfs_inorder_iter()
            .enumerate()
            .map(|(i, element)| [i as f64, element.range.len() as f64])
            .twice_iter(),
    );

    fb.finish_plot(splot, move_format!("query_num_per_node_theory_{}", prefix));
}
pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    handle2(fb, "default", DEFAULT_GROW, num_bots);
    handle2(fb, "dense", MEGA_DENSE_GROW, num_bots);
    handle2(fb, "sparse", SPARSE_GROW, num_bots);
}
