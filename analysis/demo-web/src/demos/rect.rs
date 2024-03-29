use super::*;

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let mut bots = support::make_rand_rect(dim, [5.0, 20.0])
        .take(200)
        .map(|rect| bbox(rect.inner_as::<i32>(), ()))
        .collect::<Vec<_>>();

    let mut verts = Vec::new();

    let mut a = simple2d::shapes(&mut verts);
    for bot in bots.iter() {
        a.rect(bot.rect.inner_as());
    }
    let rect_save = ctx.buffer_static_clear(&mut verts);

    let mut buffer = ctx.buffer_dynamic();

    let tree_data = broccoli::Tree::new(&mut bots).get_tree_data();

    move |data| {
        let DemoData {
            cursor, sys, ctx, ..
        } = data;

        let mut cam = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let mut tree = broccoli::Tree::from_tree_data(&mut bots, &tree_data);

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

        let mut to_draw = simple2d::shapes(&mut verts);

        tree.find_all_in_rect(AabbPin::new(&mut r1), |_, a| {
            to_draw.rect(a.rect.inner_as());
        });

        let res = if !r1.intersects_rect(&r2) {
            tree.find_all_in_rect(AabbPin::new(&mut r2), |_, a| {
                to_draw.rect(a.rect.inner_as());
            });
            true
        } else {
            false
        };

        let col = if res {
            to_draw.rect(r1.inner_as());
            to_draw.rect(r2.inner_as());
            &[0.0, 1.0, 0.0, 0.5]
        } else {
            to_draw.rect(r1.inner_as());
            to_draw.rect(r2.inner_as());
            &[1.0, 0.0, 0.0, 0.5]
        };

        buffer.update_clear(&mut verts);

        ctx.draw_clear([0.0; 4]);

        cam.draw_triangles(&rect_save, &[0.3, 0.3, 0.3, 0.4]);
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
