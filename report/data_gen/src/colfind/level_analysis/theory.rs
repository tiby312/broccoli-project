use super::*;

struct Res {
    rebal: Vec<usize>,
    query: Vec<usize>,
}

impl Res{
    fn new(num_bots: usize, grow_iter: impl Iterator<Item = f64>) -> Vec<Res> {
        let mut rects = Vec::new();
        for grow in grow_iter {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();
    
            let (rebal, query) = datanum::datanum_test2(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));
    
                let (mut tree, levelc) =
                    TreeBuilder::new(&mut bots).build_with_splitter_seq(LevelCounter::new());
    
                maker.reset();
                let levelc2 = tree.new_builder().query_with_splitter_seq(
                    |a, b| {
                        a.unpack_inner().x += 1.0;
                        b.unpack_inner().y += 1.0;
                    },
                    LevelCounter::new(),
                );
    
                (levelc.into_levels(), levelc2.into_levels())
            });
    
            let t = Res { grow, rebal, query };
    
            assert_eq!(t.rebal.len(), t.query.len());
            rects.push(t)
        }
        rects
    }
    
}

use crate::inner_prelude::*;

pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    let res1 = handle_inner_theory(
        num_bots,
        (0..100).map(|a| {
            let a: f64 = a as f64;
            0.0005 + a * 0.0001
        }),
    );

    let res2 = handle_inner_theory(
        num_bots,
        (0..100).map(|a| {
            let a: f64 = a as f64;
            0.01 + a * 0.0002
        }),
    );

    use gnuplot::*;

    fn draw_graph(title_name: &str, fg: &mut Figure, res: &[TheoryRes], rebal: bool, pos: usize) {
        let ax = fg
            .axes2d()
            .set_pos_grid(2, 1, pos as u32)
            .set_title(title_name, &[])
            .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
            .set_x_label("Spiral Grow", &[])
            .set_y_label("Number of Comparisons", &[]);

        let num = res.first().unwrap().rebal.len();

        let x = res.iter().map(|a| a.grow);

        if rebal {
            let cc = (0..num).map(|ii: usize| res.iter().map(move |a| a.rebal[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", i);
                let yl = y.clone().map(|_| 0.0);
                ax.fill_between(x.clone(), yl, y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        } else {
            let cc = (0..num).map(|ii: usize| res.iter().map(move |a| a.query[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", i);
                let yl = y.clone().map(|_| 0.0);
                ax.fill_between(x.clone(), yl, y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        }
    }

    let mut fg = fb.build("level_analysis_theory_rebal");
    draw_graph(
        &format!("Rebal Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res1,
        true,
        0,
    );
    draw_graph(
        &format!("Rebal Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res2,
        true,
        1,
    );
    fb.finish(fg);

    let mut fg = fb.build("level_analysis_theory_query");
    draw_graph(
        &format!("Query Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res1,
        false,
        0,
    );
    draw_graph(
        &format!("Query Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res2,
        false,
        1,
    );
    fb.finish(fg);
}

