use crate::support::prelude::*;
use std;

use axgeom::Ray;

#[derive(Copy, Clone)]
struct Bot {
    center: Vec2<f32>,
}

pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let radius = 10.0;

    let vv = support::make_rand(400, dim, |center| {
        bbox(Rect::from_point(center, vec2same(radius)), Bot { center })
    })
    .into_boxed_slice();

    //Draw bots
    let mut r = canvas.circles();
    for bot in vv.iter() {
        r.add(bot.inner.center.into());
    }
    let circle_save = r.save(canvas);

    let mut tree = broccoli::container::TreeOwned::new(vv);

    Demo::new(move |cursor, canvas, check_naive| {
        circle_save
            .uniforms(canvas, radius * 2.0)
            .with_color([0.0, 0.0, 0.0, 0.3])
            .draw();

        {
            let tree = tree.as_tree_mut();
            let mut ray_cast = canvas.lines(1.0);

            for dir in 0..360i32 {
                let dir = dir as f32 * (std::f32::consts::PI / 180.0);
                let x = (dir.cos() * 20.0) as f32;
                let y = (dir.sin() * 20.0) as f32;

                let ray = {
                    let k = vec2(x, y);
                    Ray {
                        point: cursor,
                        dir: k,
                    }
                };

                let mut radius = radius;

                if check_naive {
                    tree.assert_raycast_mut(
                        ray,
                        &mut radius,
                        move |_r, ray, rect| ray.cast_to_rect(rect),
                        move |r, ray, t| ray.cast_to_circle(t.inner.center, *r),
                        dim,
                    );
                }

                let res = tree.raycast_mut(
                    ray,
                    &mut radius,
                    move |_r, ray, rect| ray.cast_to_rect(rect),
                    move |r, ray, t| ray.cast_to_circle(t.inner.center, *r),
                    dim,
                );

                let dis = match res {
                    axgeom::CastResult::Hit((_, dis)) => dis,
                    axgeom::CastResult::NoHit => 800.0,
                };

                let end = ray.point_at_tval(dis);
                ray_cast.add(ray.point.into(), end.into());
            }
            ray_cast
                .send_and_uniforms(canvas)
                .with_color([1.0, 1.0, 1.0, 0.4])
                .draw();
        }
    })
}
