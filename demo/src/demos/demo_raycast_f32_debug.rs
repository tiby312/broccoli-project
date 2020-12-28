use crate::support::prelude::*;

use axgeom::Ray;

pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let walls = support::make_rand_rect(5000, dim, [1.0, 4.0], |a| bbox(a, ())).into_boxed_slice();

    let mut counter: f32 = 0.0;
    let mut tree = broccoli::container::TreeOwned::new_par(walls);

    let mut rects = canvas.rects();
    for bot in tree.as_tree().get_bbox_elements().iter() {
        rects.add(bot.rect.into());
    }
    let rect_save = rects.save(canvas);

    Demo::new(move |cursor, canvas, check_naive| {
        let tree = tree.as_tree_mut();

        let ray: Ray<f32> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            Ray { point, dir }
        };

        //Draw the walls
        rect_save
            .uniforms(canvas)
            .with_color([0.0, 0.0, 0.0, 0.3])
            .draw();


        let mut rects = canvas.rects();

        let mut handler=broccoli::query::raycast::from_closure(
            tree,
            &mut rects,
            |rects, ray, a| {rects.add(a.rect.into());ray.cast_to_rect(&a.rect)},
            |_, ray, a| ray.cast_to_rect(&a.rect),
            |_, ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
            |_, ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
        );

        if check_naive {
            broccoli::query::raycast::assert_raycast(tree,ray, &mut handler);          
        }

        let test = {
            let test = tree.raycast_mut(ray, &mut handler);
            rects
                .send_and_uniforms(canvas)
                .with_color([4.0, 0.0, 0.0, 0.4])
                .draw();
            test
        };

        let dis = match test {
            axgeom::CastResult::Hit((_, dis)) => dis,
            axgeom::CastResult::NoHit => 800.0,
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
