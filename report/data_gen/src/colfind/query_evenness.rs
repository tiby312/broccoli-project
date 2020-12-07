use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;
#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<i32>,
    num: usize,
}


type LTree=compt::dfs_order::CompleteTreeContainer<usize,compt::dfs_order::PreOrder>;


struct TheoryRes {
    grow: f32,
    query: LTree,
}

fn handle_inner_theory(num_bots: usize, grow:f32) -> TheoryRes {
    
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

    let query = datanum::datanum_test2(|maker| {
        let mut bots: Vec<BBox<_, &mut _>> = abspiral_datanum_f32_nan(maker, grow as f64)
            .zip(bot_inner.iter_mut())
            .map(|(a, b)| bbox(a, b))
            .collect();

        let mut levelc = LevelCounter::new();
        let mut tree = TreeBuilder::new(&mut bots).build_with_splitter_seq(&mut levelc);

        //let start=maker.count();
        maker.reset();

        let mut levelc2 = LevelCounter::new();
        tree.new_colfind_builder().query_with_splitter_seq(
            |a, b| {
                a.unpack_inner().x += 1.0;
                b.unpack_inner().y += 1.0;
            },
            &mut levelc2,
        );

        let mut ll=levelc2.into_tree();
        /*
        use broccoli::compt::Visitor;
        
        for a in ll.vistr_mut().dfs_preorder_iter(){
            *a-=start;
        }
        */

        ll
    });

    TheoryRes {
        grow,
        query,
    }
}



pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    let res1 = handle_inner_theory(
        num_bots,
        0.2,
    );

    let res2 = handle_inner_theory(
        num_bots,
        0.007,
    );

    use gnuplot::*;

    fn draw_graph(title_name: &str, fg: &mut Figure, res:TheoryRes, rebal: bool, pos: usize) {
        let ax = fg
            .axes2d()
            .set_pos_grid(2, 1, pos as u32)
            .set_title(title_name, &[])
            .set_x_label("dfs inorder iteration", &[])
            .set_y_label("Number of Comparisons", &[]);

            use broccoli::compt::Visitor;
        
            //height*x=num_nodes
            //
        let xx=res.query.get_nodes().len()/res.query.get_height();
        let height=res.query.get_height();
        //let num_nodes=res.query.get_nodes().len();
        for (i,(depth,element)) in res.query.vistr().with_depth(compt::Depth(0)).dfs_inorder_iter().enumerate(){
            let s=format!("depth:{}",depth.0);
            //let width=(2 as f32).powi( 1+(height-1-depth.0)as i32) as usize;
            let width=2;
            let col=COLS.iter().cycle().nth(depth.0).unwrap();
            ax.boxes_set_width(std::iter::once(i),std::iter::once(element),std::iter::once(width),&[Color(col)]);
       
        }

    }

    let mut fg = fb.build("query_evenness_theory");
    draw_graph(
        &format!("Query Evenness with {} Objects", num_bots),
        &mut fg,
        res1,
        false,
        0,
    );
    draw_graph(
        &format!("Query Evenness with {} Objects", num_bots),
        &mut fg,
        res2,
        false,
        1,
    );
    fb.finish(fg);
}
