use crate::support::prelude::*;

pub fn make_demo(dim: Rect<f32>, ctx: &web_sys::WebGl2RenderingContext) -> impl FnMut(DemoData) {
    let bots = support::make_rand_rect(dim, [5.0, 20.0])
        .take(200)
        .map(|rect| bbox(rect.inner_as::<i32>(), ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();


    let rect_save = {
        let mut verts = vec![];
        for bot in bots.iter() {
            verts.rect(bot.inner_as().rect);
        }
        ctx.buffer_static(&verts)
    };

    let mut buffer = ctx.buffer_dynamic();

    let mut tree = broccoli::container::TreeOwned::new(bots);

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        let tree = tree.as_tree_mut();

        let cc: Vec2<i32> = cursor.inner_as();
        let r1 = axgeom::Rect::new(cc.x - 100, cc.x + 100, cc.y - 100, cc.y + 100);
        let r2 = axgeom::Rect::new(100, 400, 100, 400);

        if check_naive {
            use broccoli::assert::*;

            assert_for_all_in_rect_mut(tree, &r1);
            assert_for_all_in_rect_mut(tree, &r2);
            assert_for_all_intersect_rect_mut(tree, &r1);
            assert_for_all_intersect_rect_mut(tree, &r2);
            assert_for_all_not_in_rect_mut(tree, &r1);
        }

        //test MultiRect

        let mut rects = tree.multi_rect();

        let mut to_draw = Vec::new();
        rects
            .for_all_in_rect_mut(r1, |a| to_draw.rect(a.rect.inner_as()))
            .unwrap();

        let res = rects.for_all_in_rect_mut(r2, |a| {
            to_draw.rect(a.rect.inner_as());
        });

        let mut cam = sys.camera(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let col = match res {
            Ok(()) => {
                to_draw.rect(r1.inner_as());
                to_draw.rect(r2.inner_as());
                &[0.0, 1.0, 0.0, 0.5]
            }
            Err(_) => {
                to_draw.clear();
                to_draw.rect(r1.inner_as());
                to_draw.rect(r2.inner_as());
                buffer.update(&to_draw);
                &[1.0, 0.0, 0.0, 0.5]
            }
        };

        ctx.clear_color(0.13, 0.13, 0.13, 1.0);
        ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        cam.draw_triangles(&rect_save, &[0.3, 0.3, 0.3, 0.4]);

        buffer.update(&to_draw);
        cam.draw_triangles(&buffer, col);

        ctx.flush();

        /*
        //test for_all_intersect_rect
        let mut rects = canvas.rects();
        tree.for_all_intersect_rect(&r1, |a| {
            rects.add(a.rect.inner_as().into());
        });
        rects
            .send_and_uniforms(canvas)
            .with_color([0.0, 0.0, 1.0, 0.2])
            .draw();

        canvas
            .rects()
            .add(r1.inner_as().into())
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.0, 0.0, 0.2])
            .draw();

        //test for_all_not_in_rect_mut
        let mut rects = canvas.rects();
        tree.for_all_not_in_rect_mut(&r1, |b| {
            rects.add(b.rect.inner_as().into());
        });
        rects
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.0, 0.0, 0.5])
            .draw();
        */
    }
}
