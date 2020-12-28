use crate::inner_prelude::*;

type LTree = compt::dfs_order::CompleteTreeContainer<usize, compt::dfs_order::PreOrder>;

struct TheoryRes {
    query: LTree,
}

fn handle_inner_theory(num_bots: usize, grow: f64) -> TheoryRes {
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

    let query = datanum::datanum_test2(|maker| {
        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));

        let mut levelc = LevelCounter::new();
        let mut tree = TreeBuilder::new(&mut bots).build_with_splitter_seq(&mut levelc);

        maker.reset();

        let mut levelc2 = LevelCounter::new();
        tree.new_colfind_builder().query_with_splitter_seq(
            |a, b| {
                a.unpack_inner().x += 1.0;
                b.unpack_inner().y += 1.0;
            },
            &mut levelc2,
        );

        levelc2.into_tree()
    });

    TheoryRes { query }
}

pub fn handle_num_node(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    let mut fg = fb.build("tree_num_per_node_theory");

    let grow1 = 0.2;
    let grow2 = 0.007;
    handle_tree_num_per_node(
        num_bots,
        grow1,
        &format!(
            "Num of aabbs per node with abspiral({},{})",
            num_bots, grow1
        ),
        &mut fg,
        0,
    );

    handle_tree_num_per_node(
        num_bots,
        grow2,
        &format!("Num of aabbs per node abspiral({},{})", num_bots, grow2),
        &mut fg,
        1,
    );

    fb.finish(fg);
}
pub fn handle_tree_num_per_node(
    num_bots: usize,
    grow: f64,
    title_name: &str,
    fg: &mut Figure,
    pos: usize,
) {
    use broccoli::compt::Visitor;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

    let tree = broccoli::new(&mut bots);

    let ax = fg
        .axes2d()
        .set_pos_grid(2, 1, pos as u32)
        .set_title(title_name, &[])
        .set_x_label("dfs inorder iteration", &[])
        .set_y_label("Number of aabbs", &[]);

    for (i, (depth, num)) in tree
        .vistr()
        .map(|a| a.range.len())
        .with_depth(compt::Depth(0))
        .dfs_inorder_iter()
        .enumerate()
    {
        let width = 2;
        let col = COLS.iter().cycle().nth(depth.0).unwrap();
        ax.boxes_set_width(
            std::iter::once(i),
            std::iter::once(num),
            std::iter::once(width),
            &[Color(col)],
        );
    }
}
pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    let grow1 = 0.2;
    let grow2 = 0.007;
    let res1 = handle_inner_theory(num_bots, grow1);

    let res2 = handle_inner_theory(num_bots, grow2);

    use gnuplot::*;

    fn draw_graph(title_name: &str, fg: &mut Figure, res: TheoryRes, pos: usize) {
        let ax = fg
            .axes2d()
            .set_pos_grid(2, 1, pos as u32)
            .set_title(title_name, &[])
            .set_x_label("dfs inorder iteration", &[])
            .set_y_label("Number of Comparisons", &[]);

        use broccoli::compt::Visitor;
        for (i, (depth, element)) in res
            .query
            .vistr()
            .with_depth(compt::Depth(0))
            .dfs_inorder_iter()
            .enumerate()
        {
            let width = 2;
            let col = COLS.iter().cycle().nth(depth.0).unwrap();
            ax.boxes_set_width(
                std::iter::once(i),
                std::iter::once(element),
                std::iter::once(width),
                &[Color(col)],
            );
        }
    }

    let mut fg = fb.build("query_evenness_theory");
    draw_graph(
        &format!("Query Evenness with abspiral({},{})", grow1, num_bots),
        &mut fg,
        res1,
        0,
    );
    draw_graph(
        &format!("Query Evenness with abspiral({},{})", grow2, num_bots),
        &mut fg,
        res2,
        1,
    );
    fb.finish(fg);
}
