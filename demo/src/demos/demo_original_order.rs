use crate::support::prelude::*;

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

pub fn make_demo(dim: Rect<f32>) -> Demo {
    let radius = 5.0;

    let mut bots = {
        let mut idcounter = 0;
        support::make_rand(4000, dim, |pos| {
            let b = Bot {
                id: idcounter,
                pos,
                vel: vec2same(0.0),
                force: vec2same(0.0),
            };
            idcounter += 1;
            b
        })
    };

    Demo::new(move |cursor, canvas, check_naive| {
        for b in bots.iter_mut() {
            b.update();
        }

        let mut k: Vec<_> = bots
            .iter_mut()
            .map(|b| {
                let r = Rect::from_point(b.pos, vec2same(radius));
                bbox(r, b)
            })
            .collect();

        let mut tree = broccoli::container::TreeRef::new_par(&mut k);

        {
            tree.for_all_not_in_rect_mut(&dim, |a| {
                let a = a.unpack_inner();
                duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim, 0.5);
            });
        }

        let vv = vec2same(100.0);
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |b| {
            let b = b.unpack_inner();
            let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cursor, 0.001, 20.0);
        });

        //Draw the dividers
        let mut rects = canvas.rects();

        tree.draw_divider(
            &mut rects,
            |rects, _, cont, length, _| {
                rects.add(
                    Rect {
                        x: cont.into(),
                        y: length.into(),
                    }
                    .into(),
                );
            },
            |rects, _, cont, length, _| {
                rects.add(
                    Rect {
                        x: length.into(),
                        y: cont.into(),
                    }
                    .into(),
                );
            },
            &dim,
        );
        rects
            .send_and_uniforms(canvas)
            .with_color([0.0, 1.0, 1.0, 0.6])
            .draw();

        //Draw lines to the bots.
        let mut lines = canvas.lines(2.0);
        use broccoli::query::Queries;
        draw_bot_lines(tree.axis(), tree.vistr(), &dim, &mut lines);
        lines
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.5, 1.0, 0.6])
            .draw();

        tree.find_colliding_pairs_mut_par(|a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            let _ = duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
        });

        if check_naive {
            broccoli::query::colfind::assert_query(&mut tree);
        }

        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into());
        }
        circles
            .send_and_uniforms(canvas, radius)
            .with_color([1.0, 1.0, 0.0, 0.6])
            .draw();

        let mut lines = canvas.lines(radius * 0.5);
        for bot in bots.iter() {
            lines.add(
                bot.pos.into(),
                (bot.pos + vec2(0.0, (bot.id % 100) as f32) * 0.1).into(),
            );
        }
        lines
            .send_and_uniforms(canvas)
            .with_color([0.0, 0.0, 1.0, 0.5])
            .draw();
    })
}

use broccoli::node::Node;
use broccoli::node::Vistr;

fn draw_bot_lines<A: axgeom::Axis>(
    axis: A,
    stuff: Vistr<Node<BBox<f32, &mut Bot>>>,
    rect: &axgeom::Rect<f32>,
    lines: &mut egaku2d::shapes::LineSession,
) {
    use compt::Visitor;
    let (nn, rest) = stuff.next();
    //let nn = nn.get();
    let mid = match rest {
        Some([start, end]) => match nn.div {
            Some(div) => {
                let (a, b) = rect.subdivide(axis, div);

                draw_bot_lines(axis.next(), start, &a, lines);
                draw_bot_lines(axis.next(), end, &b, lines);

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

                Some((midx, midy))
            }
            None => None,
        },
        None => {
            let ((x1, x2), (y1, y2)) = rect.get();
            let midx = x1 + (x2 - x1) / 2.0;

            let midy = y1 + (y2 - y1) / 2.0;

            Some((midx, midy))
        }
    };

    if let Some((midx, midy)) = mid {
        for b in nn.range.iter() {
            let _bx = b.inner.pos.x;
            let _by = b.inner.pos.y;
            lines.add(b.inner.pos.into(), vec2(midx, midy).into());
        }
    }
}
