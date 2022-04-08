use super::*;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    mass: f32,
}

impl Bot {
    fn handle(&mut self) {
        let b = self;

        b.pos += b.vel;

        //F=MA
        //A=F/M
        let acc = b.force / b.mass;

        b.vel += acc;

        b.force = vec2same(0.0);
    }
}

#[derive(Copy, Clone, Debug)]
struct NodeMass {
    center: Vec2<f32>,
    mass: f32,
    force: Vec2<f32>,
}

use core::default::Default;
impl Default for NodeMass {
    fn default() -> NodeMass {
        NodeMass {
            center: vec2(Default::default(), Default::default()),
            mass: Default::default(),
            force: vec2(Default::default(), Default::default()),
        }
    }
}

use broccoli::queries::nbody::*;
use core::marker::PhantomData;

//TODO remove?
#[derive(Clone, Copy)]
struct Bla<'a> {
    _num_pairs_checked: usize,
    _p: PhantomData<&'a usize>,
}
impl<'a> broccoli::tree::splitter::Splitter for Bla<'a> {
    fn div(self) -> (Self, Self) {
        (
            Bla {
                _p: PhantomData,
                _num_pairs_checked: 0,
            },
            Bla {
                _p: PhantomData,
                _num_pairs_checked: 0,
            },
        )
    }
    fn add(self, _: Self) -> Self {
        self
        //do nothing
    }
}
impl<'b> broccoli::queries::nbody::Nbody for Bla<'b> {
    type Mass = NodeMass;
    type T = BBox<f32, &'b mut Bot>;
    type N = f32;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self, a: &[Self::T]) -> Self::Mass {
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut total_mass = 0.0;

        for i in a.iter() {
            let m = i.inner.mass;
            total_mass += m;
            total_x += m * i.inner.pos.x;
            total_y += m * i.inner.pos.y;
        }

        let center = if total_mass != 0.0 {
            vec2(total_x / total_mass, total_y / total_mass)
        } else {
            vec2same(0.0)
        };
        NodeMass {
            center,
            mass: total_mass,
            force: vec2same(0.0),
        }
    }

    fn is_close_half(&self, m: &Self::Mass, line: Self::N, a: impl Axis) -> bool {
        if a.is_xaxis() {
            (m.center.x - line).abs() < 200.0
        } else {
            (m.center.y - line).abs() < 200.0
        }
    }

    fn is_close(&self, m: &Self::Mass, line: Self::N, a: impl Axis) -> bool {
        if a.is_xaxis() {
            (m.center.x - line).abs() < 400.0
        } else {
            (m.center.y - line).abs() < 400.0
        }
    }

    #[inline(always)]
    fn gravitate(&mut self, a: GravEnum<Self::T, Self::Mass>, b: GravEnum<Self::T, Self::Mass>) {
        match (a, b) {
            (GravEnum::Mass(a), GravEnum::Mass(b)) => {
                let _ = duckduckgeo::gravitate(
                    [
                        (a.center, a.mass, &mut a.force),
                        (b.center, b.mass, &mut b.force),
                    ],
                    0.0001,
                    0.004,
                );
            }
            (GravEnum::Mass(a), GravEnum::Bot(b)) | (GravEnum::Bot(b), GravEnum::Mass(a)) => {
                for b in b.iter_mut() {
                    let b = b.unpack_inner();

                    let _ = duckduckgeo::gravitate(
                        [
                            (a.center, a.mass, &mut a.force),
                            (b.pos, b.mass, &mut b.force),
                        ],
                        0.0001,
                        0.004,
                    );
                }
            }
            (GravEnum::Bot(b1), GravEnum::Bot(mut b2)) => {
                for mut a in b1.iter_mut() {
                    for b in b2.borrow_mut().iter_mut() {
                        let (a, b) = (a.borrow_mut().unpack_inner(), b.unpack_inner());

                        let _ = duckduckgeo::gravitate(
                            [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
                            0.0001,
                            0.004,
                        );
                    }
                }
            }
        }
    }

    fn gravitate_self(&mut self, a: TreePin<&mut [Self::T]>) {
        broccoli::queries::nbody::naive_nbody_mut(a, |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());

            let _ = duckduckgeo::gravitate(
                [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
                0.0001,
                0.004,
            );
        })
    }

    fn apply_a_mass<'a>(
        &'a mut self,
        a: Self::Mass,
        b: impl Iterator<Item = TreePin<&'a mut Self::T>>,
        len: usize,
    ) {
        if a.mass > 0.000_000_1 {
            let indforce = vec2(a.force.x / len as f32, a.force.y / len as f32);

            //TODO counteract the added fudge here, but dividing by 3 on bot on bot cases
            let fudge = 3.0;
            for i in b {
                let i = i.unpack_inner();
                let forcex = indforce.x * fudge;
                let forcey = indforce.y * fudge;

                i.force += vec2(forcex, forcey);
            }
        }
    }

    fn combine_two_masses(&mut self, a: &Self::Mass, b: &Self::Mass) -> Self::Mass {
        NodeMass {
            center: (a.center + b.center) / 2.0,
            mass: a.mass + b.mass,
            force: vec2same(0.0),
        }
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let mut bots: Vec<_> = support::make_rand(dim)
        .take(1000)
        .map(|pos| Bot {
            mass: 100.0,
            pos: pos.into(),
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    //Make one of the bots have a lot of mass.
    bots.last_mut().unwrap().mass = 10000.0;

    let mut no_mass_bots: Vec<Bot> = Vec::new();

    let mut verts = vec![];

    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        let no_mass_bots = &mut no_mass_bots;

        let mut k = support::distribute(&mut bots, |b| {
            let radius = 5.0f32.min(b.mass.sqrt() / 10.0);
            support::point_to_rect_f32(b.pos, radius)
        });

        {
            /* naive
            broccoli::query::nbody::naive_mut(PMut::new(&mut k),|a,b|{
                let (a, b) = (a.unpack_inner(), b.unpack_inner());

                let _ = duckduckgeo::gravitate(
                    [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
                    0.0001,
                    0.004,
                );
            });
            */

            //use std::time::{Duration, Instant};
            //let now = Instant::now();
            let tree = broccoli::tree::new(&mut k);

            let mut tree = broccoli::queries::nbody::nbody_mut(
                tree,
                &mut Bla {
                    _num_pairs_checked: 0,
                    _p: PhantomData,
                },
            );

            //println!("{}", now.elapsed().as_millis());
            //panic!();
            if check_naive {

                /* TODO update
                let mut bla = Bla {
                    num_pairs_checked: 0,
                    _p: PhantomData,
                };
                tree.nbody_mut(&mut bla, border);




                for b in bots3.iter_mut() {
                    b.force = vec2same(0.0);
                }

                {
                    let mut max_diff = None;

                    for (a, bb) in bots3.iter().zip(bots2.iter()) {
                        let b = &bb.inner;

                        let dis_sqr1 = a.force.magnitude2();
                        let dis_sqr2 = b.force.magnitude2();
                        let dis1 = dis_sqr1.sqrt();
                        let dis2 = dis_sqr2.sqrt();

                        let acc_dis1 = dis1 / a.mass;
                        let acc_dis2 = dis2 / a.mass;

                        let diff = (acc_dis1 - acc_dis2).abs();

                        let error: f32 = (acc_dis2 - acc_dis1).abs() / acc_dis2;

                        match max_diff {
                            None => max_diff = Some((diff, bb, error)),
                            Some(max) => {
                                if diff > max.0 {
                                    max_diff = Some((diff, bb, error))
                                }
                            }
                        }
                    }
                    /*
                    let max_diff = max_diff.unwrap();
                    self.max_percentage_error = max_diff.2 * 100.0;

                    let f = {
                        let a: f32 = num_pair_alg as f32;
                        let b: f32 = num_pair_naive as f32;
                        a / b
                    };

                    println!("absolute acceleration err={:06.5} percentage err={:06.2}% current bot not checked ratio={:05.2}%",max_diff.0,self.max_percentage_error,f*100.0);
                    */
                    //draw_rect_f32([1.0, 0.0, 1.0, 1.0], max_diff.1.get().as_ref(), c, g);
                }
                */
            }

            tree.colliding_pairs(|a, b| {
                let (a, b) = (a.unpack_inner(), b.unpack_inner());
                let (a, b) = if a.mass > b.mass { (a, b) } else { (b, a) };

                if b.mass != 0.0 {
                    let ma = a.mass;
                    let mb = b.mass;
                    let ua = a.vel;
                    let ub = b.vel;

                    //Do perfectly inelastic collision.
                    let vx = (ma * ua.x + mb * ub.x) / (ma + mb);
                    let vy = (ma * ua.y + mb * ub.y) / (ma + mb);
                    assert!(!vx.is_nan() && !vy.is_nan());
                    a.mass += b.mass;

                    a.force += b.force;
                    a.vel = vec2(vx, vy);

                    b.mass = 0.0;
                    b.force = vec2same(0.0);
                    b.vel = vec2same(0.0);
                    b.pos = vec2same(0.0);
                }
            });
        }

        //Draw bots.
        verts.clear();
        for bot in k.iter() {
            verts.rect(bot.rect);
        }
        buffer.update(&verts);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0])
            .draw_triangles(&buffer, &[1.0, 0.0, 1.0, 1.0]);

        ctx.flush();

        //Remove bots that have no mass, and add them to the pool
        //of bots that don't exist yet.
        {
            let mut new_bots = Vec::new();
            for b in bots.drain(..) {
                if b.mass == 0.0 {
                    no_mass_bots.push(b);
                } else {
                    new_bots.push(b);
                }
            }
            bots.append(&mut new_bots);
        };

        //Update bot locations.
        for bot in bots.iter_mut() {
            Bot::handle(bot);
            duckduckgeo::wrap_position(&mut bot.pos, dim);
        }

        //Add one bott each iteration.
        if let Some(mut b) = no_mass_bots.pop() {
            b.mass = 30.0;
            b.pos = cursor;
            b.force = vec2same(0.0);
            b.vel = vec2(1.0, 0.0);
            bots.push(b);
        }
    }
}
