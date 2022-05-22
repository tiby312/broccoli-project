use super::*;

use axgeom::Ray;
use broccoli::Assert;

struct MyRaycast {
    radius: f32,
}
impl<'a> broccoli::queries::raycast::RayCast<ManySwapBBox<f32, Vec2<f32>>> for MyRaycast {
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
        ray: &Ray<f32>,
        a: AabbPin<&mut ManySwapBBox<f32, Vec2<f32>>>,
    ) -> Option<axgeom::CastResult<f32>> {
        Some(ray.cast_to_rect(a.get()))
    }

    fn cast_fine(
        &mut self,
        ray: &Ray<f32>,
        a: AabbPin<&mut ManySwapBBox<f32, Vec2<f32>>>,
    ) -> axgeom::CastResult<f32> {
        ray.cast_to_circle(a.1, self.radius)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let radius = 10.0;
    let line_width = 1.0;

    let mut centers: Vec<_> = support::make_rand(dim)
        .take(200)
        .map(|x| {
            let x: Vec2<f32> = x.into();
            ManySwapBBox(Rect::from_point(x, vec2same(radius)), x)
        })
        .collect();

    let circle_save = {
        let mut f = vec![];
        for &b in centers.iter() {
            let k: [f32; 2] = b.1.into();
            f.push(k);
        }
        ctx.buffer_static(&f)
    };

    let tree_data = broccoli::Tree::new(&mut centers).get_tree_data();

    let mut verts = vec![];
    let mut buffer = ctx.buffer_dynamic();

    let mut handler = MyRaycast { radius };

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        verts.clear();

        if check_naive {
            for dir in 0..1000i32 {
                let dir = (dir as f32) * (std::f32::consts::TAU / 1000.0);
                let x = (dir.cos() * 20.0) as f32;
                let y = (dir.sin() * 20.0) as f32;

                let ray = {
                    let k = vec2(x, y);
                    Ray {
                        point: cursor,
                        dir: k,
                    }
                };

                Assert::new(&mut centers.clone()).assert_raycast(ray, &mut handler);
            }
        }

        let mut tree = broccoli::Tree::from_tree_data(&mut centers, &tree_data);

        for dir in 0..1000i32 {
            let dir = (dir as f32) * (std::f32::consts::TAU / 1000.0);
            let x = (dir.cos() * 20.0) as f32;
            let y = (dir.sin() * 20.0) as f32;

            let ray = {
                let k = vec2(x, y);
                Ray {
                    point: cursor,
                    dir: k,
                }
            };

            let res = tree.cast_ray(ray, &mut handler);

            let mag = match res {
                axgeom::CastResult::Hit(res) => res.mag,
                axgeom::CastResult::NoHit => 800.0,
            };

            let end = ray.point_at_tval(mag);

            verts.line(line_width, end, ray.point);
        }

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        buffer.update(&verts);

        let mut cam = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        cam.draw_circles(&circle_save, radius * 2.0, &[1.0, 0.0, 1.0, 1.0]);

        cam.draw_triangles(&buffer, &[0.0, 1.0, 1.0, 0.2]);

        ctx.flush();
    }
}
