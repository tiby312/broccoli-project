use crate::support::prelude::*;
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


pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let radius = 5.0;

    let mut bots=support::make_rand(4000,dim,|pos|Bot {
        pos,
        vel: vec2same(0.0),
        force: vec2same(0.0),
        wall_move: [None; 2],
    });

    let mut walls=support::make_rand_rect(10,dim,[10.0,60.0],|a|a);
    
    let mut rects = canvas.rects();
    for wall in walls.iter() {
        rects.add(wall.inner_into().into());
    }
    let rect_save = rects.save(canvas);

    Demo::new(move |cursor, canvas, _check_naive| {
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

        let mut k=support::distribute(&mut bots,|b|support::point_to_rect_f32(b.pos,radius));

        {
            
            let mut tree = broccoli::new_par(&mut k);

            tree.intersect_with_mut(&mut walls, |bot2, wall| {
                let (rect,bot) = bot2.unpack();
                let wall = wall.unpack_rect();

                let fric = 0.8;

                let wallx = &wall.x;
                let wally = &wall.y;
                let vel = bot.vel;

                let ret = match duckduckgeo::collide_with_rect(
                    &rect,
                    &wall,
                )
                .unwrap()
                {
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
            });

            let cc = cursor.inner_into();
            tree.for_all_in_rect_mut(
                &axgeom::Rect::from_point(cc, vec2same(100.0)),
                |b| {
                    let b = b.unpack_inner();
                    let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cc, 0.001, 20.0);
                },
            );

            tree.find_colliding_pairs_mut_par(|a, b| {
                let a = a.unpack_inner();
                let b = b.unpack_inner();
                let _ =
                    duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
            });
        }

        rect_save
            .uniforms(canvas)
            .with_color([0.7, 0.7, 0.7, 0.3])
            .draw();

        let mut circles = canvas.circles();
        for bot in k.iter() {
            circles.add(bot.inner.pos.into());
        }
        circles
            .send_and_uniforms(canvas, radius)
            .with_color([1.0, 0.0, 0.5, 0.3])
            .draw();
    })
}
