use crate::support::prelude::*;

use axgeom::Ray;

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let walls = support::make_rand_rect(dim, [1.0, 4.0])
        .take(2000)
        .map(|a| bbox(a, ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let mut counter: f32 = 0.0;
    let mut tree = broccoli::container::TreeOwned::new(walls);

    let rect_save = {
        let mut verts = vec![];
        for bot in tree.as_tree().get_elements().iter() {
            verts.rect(bot.rect);
        }
        ctx.buffer_static(&verts)
    };

    let mut buffer = ctx.buffer_dynamic();
    let mut verts = vec![];

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        let tree = tree.as_tree_mut();

        let ray: Ray<f32> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            Ray { point, dir }
        };

        //Draw the walls
        verts.clear();

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut cam = sys.camera(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        cam.draw_triangles(&rect_save, &[0.0, 0.0, 0.0, 0.3]);

        let mut handler = broccoli::helper::raycast_from_closure(
            tree,
            (),
            |_, _, _| None,
            |_, ray, a| {
                verts.rect(a.rect);
                ray.cast_to_rect(&a.rect)
            },
            |_, ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
            |_, ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
        );

        if check_naive {
            broccoli::assert::assert_raycast(tree, ray, &mut handler);
        }

        let test = {
            let test = tree.raycast_mut(ray, &mut handler);
            drop(handler);

            buffer.update(&verts);
            cam.draw_triangles(&buffer, &[4.0, 0.0, 0.0, 0.4]);
            test
        };

        let mag = match test {
            axgeom::CastResult::Hit(res) => res.mag,
            axgeom::CastResult::NoHit => 800.0,
        };

        let end = ray.point_at_tval(mag);

        verts.clear();
        verts.line(2.0, ray.point, end);
        buffer.update(&verts);
        cam.draw_triangles(&buffer, &[1.0, 1.0, 1.0, 0.2]);

        ctx.flush();
    }
}
