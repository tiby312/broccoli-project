use super::*;

use axgeom::Ray;
use broccoli::Assert;

struct MyRaycast {
    verts: Vec<Rect<f32>>,
}
impl broccoli::queries::raycast::RayCast<BBox<f32, ()>> for MyRaycast {
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<f32>,
        line: A,
        val: f32,
    ) -> axgeom::CastResult<f32> {
        ray.cast_to_aaline(line, val)
    }

    fn cast_broad(
        &mut self,
        _ray: &Ray<f32>,
        _a: AabbPin<&mut BBox<f32, ()>>,
    ) -> Option<axgeom::CastResult<f32>> {
        None
    }

    fn cast_fine(
        &mut self,
        ray: &Ray<f32>,
        a: AabbPin<&mut BBox<f32, ()>>,
    ) -> axgeom::CastResult<f32> {
        self.verts.push(a.rect);
        ray.cast_to_rect(&a.rect)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let walls = support::make_rand_rect(dim, [1.0, 4.0])
        .take(2000)
        .map(|a| bbox(a, ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let mut counter: f32 = 0.0;
    let mut tree = broccoli::TreeOwned::new(walls);

    let rect_save = {
        let mut verts = vec![];
        for bot in tree.as_slice_mut().iter() {
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

        let ray: Ray<f32> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            Ray { point, dir }
        };

        let mut handler = MyRaycast { verts: vec![] };

        if check_naive {
            Assert::new(&mut tree.container_ref().clone()).assert_raycast(ray, &mut handler);
        }

        let mut tree = tree.as_tree();

        //Draw the walls
        verts.clear();

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut cam = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        cam.draw_triangles(&rect_save, &[0.0, 0.0, 0.0, 0.3]);

        let test = {
            let test = tree.cast_ray(ray, &mut handler);
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
