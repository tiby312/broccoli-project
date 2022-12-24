use super::*;
use broccoli::assert::Assert;
use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    id: usize, //id used to verify pairs against naive
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
}

impl Bot {
    fn update(&mut self) {
        self.vel += self.force;
        //non linear drag
        self.vel *= 0.9;
        self.pos += self.vel;
        self.force = vec2same(0.0);
    }
}

pub fn make_demo(mut dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let radius = 5.0;

    let mut bots = {
        let mut idcounter = 0;
        support::make_rand(dim)
            .take(1000)
            .map(|pos| {
                let b = Bot {
                    id: idcounter,
                    pos: pos.into(),
                    vel: vec2same(0.0),
                    force: vec2same(0.0),
                };
                idcounter += 1;
                b
            })
            .collect::<Vec<_>>()
    };

    let mut verts = vec![];

    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        for b in bots.iter_mut() {
            b.update();
        }

        let mut tree_bots: Vec<_> = bots
            .iter_mut()
            .map(|a| (Rect::from_point(a.pos, vec2same(radius)), a))
            .collect();

        if check_naive {
            Assert::new(&mut tree_bots).assert_query();
        }

        let mut tree = broccoli::Tree::new(&mut tree_bots);

        tree.find_all_not_in_rect(AabbPin::new(&mut dim), |dim, a| {
            let a = a.unpack_inner();
            duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &*dim, 0.5);
        });

        let vv = vec2same(100.0);
        tree.find_all_in_rect(
            AabbPin::new(&mut axgeom::Rect::from_point(cursor, vv)),
            |_, b| {
                let b = b.unpack_inner();
                let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cursor, 0.001, 20.0);
            },
        );

        let mut verts2 = vec![];
        let mut j = shogo::simple2d::shapes(&mut verts);
        let mut k = shogo::simple2d::shapes(&mut verts2);
        tree.draw_divider(
            |axis, node, rect, _| {
                use AxisDyn::*;

                if !node.range.is_empty() {
                    let r = match axis {
                        X => Rect {
                            x: node.cont.into(),
                            y: rect.y.into(),
                        },
                        Y => Rect {
                            x: rect.x.into(),
                            y: node.cont.into(),
                        },
                    };

                    j.rect(r);
                }

                let mid = if let Some(div) = node.div {
                    match axis {
                        X => get_nonleaf_mid(XAXIS, rect, div),
                        Y => get_nonleaf_mid(YAXIS, rect, div),
                    }
                } else {
                    get_leaf_mid(rect)
                };

                for b in node.range.iter() {
                    k.line(1.0, b.1.pos, mid);
                }
            },
            dim,
        );

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut camera = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);
        buffer.update_clear(&mut verts);
        camera.draw_triangles(&buffer, &[0.0, 1.0, 1.0, 0.3]);
        buffer.update_clear(&mut verts2);
        camera.draw_triangles(&buffer, &[0.0, 1.0, 1.0, 0.3]);

        tree.find_colliding_pairs(|a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            let _ = duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
        });

        for bot in bots.iter() {
            verts.push(bot.pos.into());
        }
        buffer.update_clear(&mut verts);
        camera.draw_circles(&buffer, radius, &[1.0, 1.0, 0.0, 0.6]);

        let mut j = simple2d::shapes(&mut verts);
        for bot in bots.iter() {
            j.line(
                radius * 0.5,
                bot.pos,
                bot.pos + vec2(0.0, (bot.id % 100) as f32) * 0.1,
            );
        }
        buffer.update_clear(&mut verts);
        camera.draw_triangles(&buffer, &[0.0, 0.0, 0.0, 0.7]);

        ctx.flush();
    }
}

fn get_leaf_mid(rect: &Rect<f32>) -> Vec2<f32> {
    let ((x1, x2), (y1, y2)) = rect.get();
    let midx = x1 + (x2 - x1) / 2.0;

    let midy = y1 + (y2 - y1) / 2.0;
    vec2(midx, midy)
}

fn get_nonleaf_mid(axis: impl Axis, rect: &Rect<f32>, div: f32) -> Vec2<f32> {
    let ((x1, x2), (y1, y2)) = rect.get();
    let midx = if !axis.is_xaxis() {
        x1 + (x2 - x1) / 2.0
    } else {
        div
    };

    let midy = if axis.is_xaxis() {
        y1 + (y2 - y1) / 2.0
    } else {
        div
    };
    vec2(midx, midy)
}
