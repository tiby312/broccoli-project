use crate::inner_prelude::*;

type LTree = compt::dfs_order::CompleteTreeContainer<usize, compt::dfs_order::PreOrder>;

struct TheoryRes {
    query: LTree,
}
impl TheoryRes{
    fn new(num_bots:usize,grow:f64)->TheoryRes{
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

        let query = datanum::datanum_test2(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));
    
            let mut tree = TreeBuilder::new(&mut bots).build_seq();
    
            maker.reset();
    
            let levelc2 = tree.new_builder().query_with_splitter_seq(
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




pub fn handle2(fb:&mut FigureBuilder,grow:f64,num_bots:usize){
    {
        let res = TheoryRes::new(num_bots, grow);

        let mut splot=poloto::plot(&format!("Complexity of query evenness with abspiral({},{})", num_bots, grow),
            "dfs inorder iteration",
            "Number of comparisons"
        );

        splot.histogram("query",res.query
            .vistr()
            .dfs_inorder_iter()
            .enumerate().map(|(i,  element)|{
                [i as f32,*element as f32]
            }));

        fb.finish_plot(splot,&format!("query_evenness_theory_{}",grow));
    }

    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

    let tree = broccoli::new(&mut bots);

    let mut splot=poloto::plot(&format!("Num per node with abspiral({},{})", num_bots, grow),
        "DFS inorder iteration",
        "Number of comparisons"
    );

    use broccoli::compt::Visitor;
    splot.histogram("query",tree
        .vistr()
        .dfs_inorder_iter()
        .enumerate().map(|(i,  element)|{
            [i as f32,element.range.len() as f32]
        }));

    fb.finish_plot(splot,&format!("query_num_per_node_theory_{}",grow));

}
pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    
    handle2(fb,0.2,num_bots);
    handle2(fb,0.007,num_bots);
    handle2(fb,2.0,num_bots);
    
    
}
