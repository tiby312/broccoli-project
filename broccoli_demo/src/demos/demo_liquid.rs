use crate::support::prelude::*;
use duckduckgeo;

use axgeom::Rect;

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
        let damping_ratio = 0.00027;
        let spring_dampen = velociy_diff.dot(diff) * (1. / dis) * damping_ratio;

        let spring_force = diff * (1. / dis) * (spring_force_mag + spring_dampen);

        self.acc += spring_force;
        b.acc -= spring_force;

        spring_force_mag
    }
}
pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let radius = 50.0;
    let mut bots: Vec<_> = dists::rand2_iter(dim.inner_into())
        .take(2000)
        .map(|[x,y]| Liquid::new(vec2(x,y)))
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {
        let mut k: Vec<_> = bots
            .iter_mut()
            .map(|bot| {
                let p = bot.pos;
                let r = radius;
                let rect = Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
                    .inner_try_into::<NotNan<f32>>()
                    .unwrap();
                bbox(rect, bot)
            })
            .collect();

        let mut tree = DinoTree::new_par(&mut k);

        tree.find_intersections_mut_par(move |a, b| {
            let _ = a.solve(b, radius);
        });

        let vv = vec2same(100.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();

        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), move |b| {
            let _ = duckduckgeo::repel_one(b.pos, &mut b.acc, cc, 0.001, 100.0);
        });

        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, move |a| {
                duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim2, 0.5);
            });
        }

        for b in bots.iter_mut() {
            b.pos += b.vel;
            b.vel += b.acc;
            b.acc = vec2same(0.0);
        }

        let mut circle = canvas.circles();
        for bot in bots.iter() {
            circle.add(bot.pos.into());
        }
        circle
            .send_and_uniforms(canvas, 2.0)
            .with_color([1.0, 0.6, 0.7, 0.5])
            .draw();
    })
}
