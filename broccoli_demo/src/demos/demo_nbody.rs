use crate::support::prelude::*;

use duckduckgeo;

#[derive(Copy, Clone)]
struct NodeMass {
    rect: axgeom::Rect<F32n>,
    center: Vec2<f32>,
    mass: f32,
    force: Vec2<f32>,
}

use core::marker::PhantomData;

#[derive(Clone, Copy)]
struct Bla<'a> {
    _num_pairs_checked: usize,
    _p: PhantomData<&'a usize>,
}
impl<'b> NodeMassTrait for Bla<'b> {
    type No = NodeMass;
    type Item = BBox<F32n, &'b mut Bot>;
    type Num = F32n;

    fn get_rect(a: &Self::No) -> &axgeom::Rect<F32n> {
        &a.rect
    }

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self, a: &mut Self::No, b: &mut Self::No) {
        let _ = duckduckgeo::gravitate(
            [
                (a.center, a.mass, &mut a.force),
                (b.center, b.mass, &mut b.force),
            ],
            0.0001,
            0.004,
        );
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self, a: &mut &'b mut Bot, b: &mut &'b mut Bot) {
        let _ = duckduckgeo::gravitate(
            [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
            0.0001,
            0.004,
        );
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self, a: &mut Self::No, b: &mut &'b mut Bot) {
        let _ = duckduckgeo::gravitate(
            [
                (a.center, a.mass, &mut a.force),
                (b.pos, b.mass, &mut b.force),
            ],
            0.0001,
            0.004,
        );
    }

    fn new<'a, I: Iterator<Item = &'a Self::Item>>(
        &'a self,
        it: I,
        rect: axgeom::Rect<F32n>,
    ) -> Self::No {
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut total_mass = 0.0;

        for i in it {
            let m = i.inner().mass;
            total_mass += m;
            total_x += m * i.inner().pos.x;
            total_y += m * i.inner().pos.y;
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
            rect,
        }
    }

    fn apply_to_bots<'a, I: Iterator<Item = &'a mut &'b mut Bot>>(
        &'a self,
        a: &'a Self::No,
        it: I,
    ) {
        if a.mass > 0.000_000_1 {
            let total_forcex = a.force.x;
            let total_forcey = a.force.y;

            for i in it {
                let forcex = total_forcex * (i.mass / a.mass);
                let forcey = total_forcey * (i.mass / a.mass);

                i.force += vec2(forcex, forcey);
            }
        }
    }

    fn is_far_enough(&self, b: [F32n; 2]) -> bool {
        (b[0].into_inner() - b[1].into_inner()).abs() > 200.0
    }

    fn is_far_enough_half(&self, b: [F32n; 2]) -> bool {
        (b[0].into_inner() - b[1].into_inner()).abs() > 100.0
    }
}

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
    fn create_aabb(&self) -> axgeom::Rect<F32n> {
        let r = 5.0f32.min(self.mass.sqrt() / 10.0);
        axgeom::Rect::from_point(self.pos, vec2same(r))
            .inner_try_into()
            .unwrap()
    }
}

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let mut bots: Vec<_> = dists::rand2_iter(dim.inner_into())
        .take(4000)
        .map(|[x,y]| Bot {
            mass: 100.0,
            pos:vec2(x,y),
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    //Make one of the bots have a lot of mass.
    bots.last_mut().unwrap().mass = 10000.0;

    let mut no_mass_bots: Vec<Bot> = Vec::new();

    Demo::new(move |cursor, canvas, check_naive| {
        let no_mass_bots = &mut no_mass_bots;
        let bots = &mut bots;

        let mut k: Vec<_> = bots.iter_mut().map(|b| bbox(b.create_aabb(), b)).collect();

        //let mut k = bbox_helper::create_bbox_mut(bots, |b| b.create_aabb());

        {
            let mut tree = DinoTree::new_par(&mut k);

            let border = dim;

            tree.nbody_mut(
                &mut Bla {
                    _num_pairs_checked: 0,
                    _p: PhantomData,
                },
                border,
            );

            if check_naive {

                /*
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

            tree.find_intersections_mut_par(|a, b| {
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

            if check_naive {
                //TODO
            }
        }
        //Draw bots.
        let mut rects = canvas.rects();
        for bot in k.iter() {
            rects.add(bot.rect.inner_into().into());
        }
        rects
            .send_and_uniforms(canvas)
            .with_color([0.9, 0.9, 0.3, 0.6])
            .draw();

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
            duckduckgeo::wrap_position(&mut bot.pos, *dim.as_ref());
        }

        if let Some(mut b) = no_mass_bots.pop() {
            b.mass = 30.0;
            b.pos = cursor.inner_into();
            b.force = vec2same(0.0);
            b.vel = vec2(1.0, 0.0);
            bots.push(b);
        }
    })
}
