use super::*;

use axgeom::Ray;
use broccoli::assert::Assert;

struct MyRaycast {
    verts: Vec<Rect<f32>>,
    ray:Ray<f32>
}
impl broccoli::queries::raycast::RayCast<BBox<f32, ()>> for MyRaycast {
    fn source(&self)->[&f32;2]{
        [&self.ray.point.x,&self.ray.point.y]
    }
    fn cast_to_aaline<A: Axis>(
        &mut self,
        line: A,
        val: f32,
    ) -> axgeom::CastResult<f32> {
        self.ray.cast_to_aaline(line, val)
    }

    fn cast_broad(
        &mut self,
        _a: &BBox<f32, ()>,
    ) -> Option<axgeom::CastResult<f32>> {
        None
    }

    fn cast_fine(
        &mut self,
        a: &BBox<f32, ()>,
    ) -> axgeom::CastResult<f32> {
        self.verts.push(a.rect);
        self.ray.cast_to_rect(&a.rect)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let mut walls = support::make_rand_rect(dim, [1.0, 4.0])
        .take(2000)
        .map(|a| bbox(a, ()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let mut counter: f32 = 0.0;
    let tree_data = broccoli::Tree::new(&mut walls).get_tree_data();

    let rect_save = {
        let mut verts = vec![];
        for bot in walls.iter() {
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

        let mut handler = MyRaycast { verts: vec![],ray};

        if check_naive {

            let mut handler2 = MyRaycast { verts: vec![],ray};

            Assert::new(&mut walls.clone()).assert_raycast(  handler2);
        }

        let mut tree = broccoli::Tree::from_tree_data(&mut walls, &tree_data);

        //TODO make something like this!!!!
        // verts.acc_and_render(|acc|{

        // });

        //Draw the walls
        verts.clear();

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut cam = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        cam.draw_triangles(&rect_save, &[0.0, 0.0, 0.0, 0.3]);

        let (_,test) = {
            let test = tree.cast_ray(  handler);
            //drop(handler);

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
