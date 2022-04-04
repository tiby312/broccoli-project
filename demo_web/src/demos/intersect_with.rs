use crate::support::prelude::*;
use broccoli::tree::halfpin::HalfPin;
use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    wall_move: [Option<(f32, f32)>; 2],
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

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let radius = 5.0;

    let mut bots = support::make_rand(dim)
        .take(2000)
        .map(|pos| Bot {
            pos: pos.into(),
            vel: vec2same(0.0),
            force: vec2same(0.0),
            wall_move: [None; 2],
        })
        .collect::<Vec<_>>();

    let mut walls = support::make_rand_rect(dim, [10.0, 60.0])
        .take(10)
        .collect::<Vec<_>>();

    let rect_save = {
        let mut verts = vec![];
        for &wall in walls.iter() {
            verts.rect(wall);
        }
        ctx.buffer_static(&verts)
    };

    let mut verts = vec![];
    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor, sys, ctx, ..
        } = data;

        for b in bots.iter_mut() {
            b.update();

            if let Some((pos, vel)) = b.wall_move[0] {
                b.pos.x = pos;
                b.vel.x = vel;
            }

            if let Some((pos, vel)) = b.wall_move[1] {
                b.pos.y = pos;
                b.vel.y = vel;
            }

            b.wall_move[0] = None;
            b.wall_move[1] = None;

            duckduckgeo::wrap_position(&mut b.pos, dim);
        }
        bots[0].pos = cursor;

        let mut k = support::distribute(&mut bots, |b| support::point_to_rect_f32(b.pos, radius));

        {
            let mut tree = broccoli::tree::new_par(&mut k);

            tree.intersect_with_iter_mut(
                HalfPin::new(walls.as_mut_slice()).iter_mut(),
                |bot2, wall| {
                    //TODO borrow instead
                    let rect = bot2.rect;
                    let bot = bot2.unpack_inner();
                    let wall = wall;

                    let fric = 0.8;

                    let wallx = &wall.x;
                    let wally = &wall.y;
                    let vel = bot.vel;

                    let ret = match duckduckgeo::collide_with_rect(&rect, &wall).unwrap() {
                        duckduckgeo::WallSide::Above => {
                            [None, Some((wally.start - radius, -vel.y * fric))]
                        }
                        duckduckgeo::WallSide::Below => {
                            [None, Some((wally.end + radius, -vel.y * fric))]
                        }
                        duckduckgeo::WallSide::LeftOf => {
                            [Some((wallx.start - radius, -vel.x * fric)), None]
                        }
                        duckduckgeo::WallSide::RightOf => {
                            [Some((wallx.end + radius, -vel.x * fric)), None]
                        }
                    };
                    bot.wall_move = ret;
                },
            );

            tree.for_all_in_rect_mut(
                HalfPin::new(&mut axgeom::Rect::from_point(cursor, vec2same(100.0))),
                |_, b| {
                    let b = b.unpack_inner();
                    let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cursor, 0.001, 20.0);
                },
            );

            tree.colliding_pairs_par(|a, b| {
                let a = a.unpack_inner();
                let b = b.unpack_inner();
                let _ =
                    duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
            });
        }

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut camera = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        camera.draw_triangles(&rect_save, &[0.7, 0.7, 0.7, 0.3]);

        verts.clear();
        for bot in k.iter() {
            verts.push(bot.inner.pos.into());
        }
        buffer.update(&verts);
        camera.draw_circles(&buffer, radius, &[1.0, 0.0, 0.5, 0.3]);

        ctx.flush();
    }
}
