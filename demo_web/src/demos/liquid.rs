use crate::support::prelude::*;

use axgeom::Rect;

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let radius = 50.0;

    let mut bots: Vec<_> = dists::grid_rect_iter(2000, dim)
        .map(|a| Liquid::new(a.into()))
        .collect();

    let mut verts = vec![];

    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor, sys, ctx, ..
        } = data;

        let mut k = support::distribute(&mut bots, |bot| {
            let p = bot.pos;
            let r = radius;
            Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
        });

        let mut tree = broccoli::new(&mut k);

        /*
        broccoli::naive::query_naive_mut(broccoli::pmut::PMut::new(&mut k),
            move |a, b| {
                let (a, b) = (a.unpack_inner(), b.unpack_inner());
                let _ = a.solve(b, radius);
            }
        );
        */

        tree.find_colliding_pairs_mut(move |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            let _ = a.solve(b, radius);
        });

        let vv = vec2same(100.0);

        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), move |b| {
            let b = b.unpack_inner();
            let _ = duckduckgeo::repel_one(b.pos, &mut b.acc, cursor, 0.001, 100.0);
        });

        tree.for_all_not_in_rect_mut(&dim, move |a| {
            let a = a.unpack_inner();
            duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim, 0.5);
        });

        for b in bots.iter_mut() {
            b.pos += b.vel;
            b.vel += b.acc;
            b.acc = vec2same(0.0);
        }

        verts.clear();
        verts.extend(bots.iter().map(|x| <[f32; 2]>::from(x.pos)));
        buffer.update(&verts);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        sys.camera(vec2(dim.x.end, dim.y.end), [0.0, 0.0])
            .draw_squares(&buffer, 2.0, &[1.0, 0.0, 1.0, 1.0]);

        ctx.flush();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Liquid {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub acc: Vec2<f32>,
}

impl Liquid {
    pub fn new(pos: Vec2<f32>) -> Liquid {
        let z = vec2same(0.0);

        Liquid {
            pos,
            acc: z,
            vel: z,
        }
    }

    pub fn solve(&mut self, b: &mut Self, radius: f32) -> f32 {
        let diff = b.pos - self.pos;

        let dis_sqr = diff.magnitude2();

        if dis_sqr < 0.00001 {
            self.acc += vec2(1.0, 0.0);
            b.acc -= vec2(1.0, 0.0);
            return 0.0;
        }

        if dis_sqr >= (2. * radius) * (2. * radius) {
            return 0.0;
        }

        let dis = dis_sqr.sqrt();

        //d is zero if barely touching, 1 is overlapping.
        //d grows linearly with position of bots
        let d = 1.0 - (dis / (radius * 2.));

        let spring_force_mag = -(d - 0.5) * 0.02;

        let velociy_diff = b.vel - self.vel;
        let damping_ratio = 0.00025;
        let spring_dampen = velociy_diff.dot(diff) * (1. / dis) * damping_ratio;

        let spring_force = diff * (1. / dis) * (spring_force_mag + spring_dampen);

        self.acc += spring_force;
        b.acc -= spring_force;

        spring_force_mag
    }
}
