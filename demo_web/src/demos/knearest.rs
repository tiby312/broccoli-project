use crate::support::prelude::*;

fn distance_to_line(point: Vec2<f32>, axis: impl Axis, val: f32) -> f32 {
    let dis = (val - *point.get_axis(axis)).abs();
    dis * dis
}
fn distance_to_rect(rect: &Rect<f32>, point: Vec2<f32>) -> f32 {
    let dis = rect.distance_squared_to_point(point);
    let dis = match dis {
        Some(dis) => dis,
        None => {
            //If a point is insert a rect, the distance to it is zero.
            //So if multiple points are inside of a rect, its not clear the order in which
            //they should be returned.
            //So in the case that a point is in the rect, we establish our own ordering,
            //by falling back on the distance between the center of a rect and the point.
            //Since the distance between a rect and a point that is outside of the rect is
            //guaranteed to be positive, we have all the negative numbers in which to
            //apply our custom ordering for bots that are inside of the rect.

            //The main reason that we are doing this is so that there arn't
            //multiple solutions to the k_nearest problem so that we can easily
            //verify the solution against the naive implementation.

            //If you don't care about a single solution existing, you can simply return zero
            //for the cases that the point is inside of the rect.

            0.0
        }
    };
    dis
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let bots = support::make_rand_rect(dim, [1.0, 8.0])
        .take(500)
        .map(|rect| bbox(rect, ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let rect_save = {
        let mut verts = vec![];
        for bot in bots.iter() {
            verts.rect(bot.inner_as().rect);
        }
        ctx.buffer_static(&verts)
    };

    let mut tree = broccoli::container::TreeOwned::new(bots);

    let mut verts = vec![];
    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        let cols = [
            [1.0, 0.0, 0.0, 0.3], //red closest
            [0.0, 1.0, 0.0, 0.3], //green second closest
            [0.0, 0.0, 1.0, 0.3], //blue third closets
        ];

        //let mut rects = canvas.rects();
        let tree = tree.as_tree_mut();
        verts.clear();
        let mut handler = broccoli::helper::knearest_from_closure(
            tree,
            (),
            |_, _, _| None,
            |_, point, a| {
                verts.rect(a.rect);
                distance_to_rect(&a.rect, point)
            },
            |_, point, val| distance_to_line(point, axgeom::XAXIS, val),
            |_, point, val| distance_to_line(point, axgeom::YAXIS, val),
        );

        if check_naive {
            broccoli::assert::assert_k_nearest_mut(tree, cursor, 3, &mut handler);
        }

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut camera = sys.camera(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let mut vv = {
            let k = tree.k_nearest_mut(cursor, 3, &mut handler);
            drop(handler);

            buffer.update(&verts);

            camera.draw_triangles(&buffer, &[1.0, 1.0, 0.0, 0.3]);

            k
        };

        camera.draw_triangles(&rect_save, &[0.0, 0.1, 0.0, 0.3]);
        for (k, color) in vv.iter().rev().zip(cols.iter()) {
            verts.clear();
            verts.push(cursor.into());
            buffer.update(&verts);
            let radius = k[0].mag.sqrt() * 2.0;
            camera.draw_circles(&buffer, radius, color);

            verts.clear();
            for b in k.iter() {
                verts.rect(b.bot.rect);
            }
            buffer.update(&verts);
            camera.draw_triangles(&buffer, color);
        }

        ctx.flush();
    }
}
