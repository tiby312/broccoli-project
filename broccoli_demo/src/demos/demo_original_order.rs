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

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 4000;

    let radius = 5.0;

    let mut bots: Vec<_> = dists::rand2_iter(dim.inner_into())
        .take(num_bot)
        .enumerate()
        .map(|(id, [x,y])| Bot {
            id,
            pos:vec2(x,y),
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    Demo::new(move |cursor, canvas, check_naive| {
        for b in bots.iter_mut() {
            b.update();
        }

        let mut k: Vec<_> = bots
            .iter_mut()
            .map(|b| {
                let r = Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap();
                bbox(r, b)
            })
            .collect();

        let mut tree = DinoTree::new_par(&mut k);

        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, |a| {
                duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim2, 0.5);
            });
        }

        let vv = vec2same(100.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |b| {
            let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cc, 0.001, 20.0);
        });

        let rects = canvas.rects();
        let mut dd = Bla { rects };
        tree.draw(&mut dd, &dim);
        dd.rects
            .send_and_uniforms(canvas)
            .with_color([0.0, 1.0, 1.0, 0.6])
            .draw();

        //draw lines to the bots.

        let mut lines = canvas.lines(2.0);
        draw_bot_lines(tree.axis(), tree.vistr(), &dim, &mut lines);
        lines
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.5, 1.0, 0.6])
            .draw();

        tree.find_intersections_mut_par(|a, b| {
            let _ = duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
        });

        if check_naive {
            Assert::find_intersections_mut(&mut tree);
        }

        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into()); //TODO we're not testing that the bots were draw in the right order
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

struct Bla {
    rects: egaku2d::shapes::RectSession,
}
impl DividerDrawer for Bla {
    type N = F32n;
    fn draw_divider<A: axgeom::Axis>(
        &mut self,
        axis: A,
        div: F32n,
        cont: [F32n; 2],
        length: [F32n; 2],
        _depth: usize,
    ) {
        let _div = div.into_inner();

        /*
        let arr = if axis.is_xaxis() {
            [
                div as f64,
                length[0].into_inner() as f64,
                div as f64,
                length[1].into_inner() as f64,
            ]
        } else {
            [
                length[0].into_inner() as f64,
                div as f64,
                length[1].into_inner() as f64,
                div as f64,
            ]
        };
        */
        let cont = Range::new(cont[0], cont[1]).inner_into();
        let length = Range::new(length[0], length[1]).inner_into();

        //let radius = (1isize.max(5 - depth as isize)) as f64;

        let rect = if axis.is_xaxis() {
            Rect { x: cont, y: length }
        } else {
            Rect { x: length, y: cont }
        };

        self.rects.add(rect.into());

        //rectangle([0.0, 1.0, 1.0, 0.2], square, self.c.transform, self.g);
    }
}

fn draw_bot_lines<A: axgeom::Axis>(
    axis: A,
    stuff: Vistr<NodeMut<BBox<F32n, &mut Bot>>>,
    rect: &axgeom::Rect<F32n>,
    lines: &mut egaku2d::shapes::LineSession,
) {
    use compt::Visitor;
    let (nn, rest) = stuff.next();
    let nn = nn.get();
    let mid = match rest {
        Some([start, end]) => match nn.div {
            Some(div) => {
                let (a, b) = rect.subdivide(axis, *div);

                draw_bot_lines(axis.next(), start, &a, lines);
                draw_bot_lines(axis.next(), end, &b, lines);

                let ((x1, x2), (y1, y2)) = rect.inner_into::<f32>().get();
                let midx = if !axis.is_xaxis() {
                    x1 + (x2 - x1) / 2.0
                } else {
                    div.into_inner()
                };

                let midy = if axis.is_xaxis() {
                    y1 + (y2 - y1) / 2.0
                } else {
                    div.into_inner()
                };

                Some((midx, midy))
            }
            None => None,
        },
        None => {
            let ((x1, x2), (y1, y2)) = rect.inner_into::<f32>().get();
            let midx = x1 + (x2 - x1) / 2.0;

            let midy = y1 + (y2 - y1) / 2.0;

            Some((midx, midy))
        }
    };

    if let Some((midx, midy)) = mid {
        //let color_delta = 1.0 / nn.bots.len() as f32;
        for b in nn.bots.iter() {
            let _bx = b.inner.pos.x;
            let _by = b.inner.pos.y;
            lines.add(b.inner.pos.into(), vec2(midx, midy).into());
            //counter += color_delta;
        }
    }
}
