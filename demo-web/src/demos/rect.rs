use super::*;

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let bots = support::make_rand_rect(dim, [5.0, 20.0])
        .take(200)
        .map(|rect| bbox(rect.inner_as::<i32>(), ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let rect_save = {
        let mut verts = vec![];
        for bot in bots.iter() {
            verts.rect(bot.rect.inner_as());
        }
        ctx.buffer_static(&verts)
    };

    let mut buffer = ctx.buffer_dynamic();

    let mut tree = broccoli::tree::new_owned(bots);

    move |data| {
        let DemoData {
            cursor, sys, ctx, ..
        } = data;

        let mut tree = tree.as_tree();

        let cc: Vec2<i32> = cursor.inner_as();
        let mut r1 = axgeom::Rect::new(cc.x - 100, cc.x + 100, cc.y - 100, cc.y + 100);
        let mut r2 = axgeom::Rect::new(100, 400, 100, 400);

        /*
        if check_naive {
            use broccoli::queries::rect::*;

            assert_for_all_in_rect_mut(tree, &r1);
            assert_for_all_in_rect_mut(tree, &r2);
            assert_for_all_intersect_rect_mut(tree, &r1);
            assert_for_all_intersect_rect_mut(tree, &r2);
            assert_for_all_not_in_rect_mut(tree, &r1);
        }*/

        //test MultiRect

        let mut to_draw = Vec::new();
        tree.for_all_in_rect_mut(AabbPin::new(&mut r1), |_, a| {
            to_draw.rect(a.rect.inner_as())
        });

        let res = if !r1.intersects_rect(&r2) {
            tree.for_all_in_rect_mut(AabbPin::new(&mut r2), |_, a| {
                to_draw.rect(a.rect.inner_as());
            });
            true
        } else {
            false
        };

        let mut cam = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let col = if res {
            to_draw.rect(r1.inner_as());
            to_draw.rect(r2.inner_as());
            &[0.0, 1.0, 0.0, 0.5]
        } else {
            to_draw.clear();
            to_draw.rect(r1.inner_as());
            to_draw.rect(r2.inner_as());
            buffer.update(&to_draw);
            &[1.0, 0.0, 0.0, 0.5]
        };

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

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
