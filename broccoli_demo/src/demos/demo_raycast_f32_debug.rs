use crate::support::prelude::*;

use axgeom::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Bot;

pub fn make_demo(dim: Rect<F32n>, canvas: &mut SimpleCanvas) -> Demo {
    let ii: Vec<_> = dists::rand2_iter(dim.inner_into())
        .zip(dists::rand_iter(1.0,3.0))
        .take(5000)
        .map(|([x,y], radius)| bbox(Rect::from_point(vec2(x,y), vec2same(radius)).inner_try_into().unwrap(), Bot))
        .collect();

    let mut counter: f32 = 0.0;
    let mut tree = DinoTreeOwned::new_par(ii);

    let mut rects = canvas.rects();
    for bot in tree.get_bots().iter() {
        rects.add(bot.get().inner_into().into());
    }
    let rect_save = rects.save(canvas);

    Demo::new(move |cursor, canvas, check_naive| {
        let tree = tree.as_tree_mut();

        let ray: Ray<F32n> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor.inner_into::<f32>().inner_as();
            //*counter=10.0;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            let dir = dir.inner_as();
            Ray { point, dir }.inner_try_into().unwrap()
        };

        rect_save
            .uniforms(canvas)
            .with_color([0.0, 0.0, 0.0, 0.3])
            .draw();

        if check_naive {
            Assert::raycast_mut(
                tree,
                ray,
                &mut rects,
                move |_r, ray, rect| {
                    ray.inner_into::<f32>()
                        .cast_to_rect(rect.as_ref())
                        .map(|a| f32n(a))
                },
                move |rects, ray, t| {
                    rects.add(t.get().inner_into().into());
                    ray.inner_into::<f32>()
                        .cast_to_rect(t.get().as_ref())
                        .map(|a| f32n(a))
                },
                dim,
            );
        }

        let test = {
            let mut rects = canvas.rects();

            let test = tree.raycast_mut(
                ray,
                &mut rects,
                move |_r, ray, rect| {
                    ray.inner_into::<f32>()
                        .cast_to_rect(rect.as_ref())
                        .map(|a| f32n(a))
                },
                move |r, ray, d| {
                    r.add(d.get().inner_into().into());
                    
                    ray.inner_into::<f32>()
                        .cast_to_rect(d.get().as_ref())
                        .map(|a| f32n(a))
                },
                dim,
            );
            rects
                .send_and_uniforms(canvas)
                .with_color([4.0, 0.0, 0.0, 0.4])
                .draw();
            test
        };

        let ray: Ray<f32> = ray.inner_into();

        let dis = match test {
            RayCastResult::Hit((_, dis)) => dis.into_inner(),
            RayCastResult::NoHit => 800.0,
        };

        let end = ray.point_at_tval(dis);

        canvas
            .lines(2.0)
            .add(ray.point.into(), end.into())
            .send_and_uniforms(canvas)
            .with_color([1., 1., 1., 0.2])
            .draw();
    })
}
