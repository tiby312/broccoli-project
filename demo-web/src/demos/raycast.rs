use super::*;

use axgeom::Ray;

#[derive(Copy, Clone)]
struct Bot {
    center: Vec2<f32>,
}
struct MyRaycast {
    radius: f32,
}
impl broccoli::queries::raycast::RayCast<BBox<f32, Bot>> for MyRaycast {
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
        a: AabbPin<&mut BBox<f32, Bot>>,
    ) -> Option<axgeom::CastResult<f32>> {
        Some(ray.cast_to_rect(a.get()))
    }

    fn cast_fine(
        &mut self,
        ray: &Ray<f32>,
        a: AabbPin<&mut BBox<f32, Bot>>,
    ) -> axgeom::CastResult<f32> {
        ray.cast_to_circle(a.inner.center, self.radius)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let radius = 10.0;
    let line_width = 1.0;

    let vv = support::make_rand(dim)
        .take(200)
        .map(|c| {
            let center = c.into();
            bbox(Rect::from_point(center, vec2same(radius)), Bot { center })
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let circle_save = {
        let mut f = vec![];
        for b in vv.iter() {
            f.push(b.inner.center.into());
        }
        ctx.buffer_static(&f)
    };

    let mut verts = vec![];
    let mut buffer = ctx.buffer_dynamic();

    let mut tree = broccoli::tree::TreeOwned::new(vv);

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
            let mut vv_clone = tree.clone_inner();
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

                broccoli::queries::raycast::assert_raycast(&mut vv_clone, ray, &mut handler);
            }
        }

        let mut tree = tree.as_tree();

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

            let res = tree.raycast_mut(ray, &mut handler);

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