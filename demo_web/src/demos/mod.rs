pub mod liquid;
pub mod raycast;
pub mod nbody;

pub struct Demo(
    Box<dyn FnMut(Vec2<f32>, &mut shogo::dots::ShaderSystem, &WebGl2RenderingContext, bool)>,
);
impl Demo {
    pub fn new(
        func: impl FnMut(Vec2<f32>, &mut shogo::dots::ShaderSystem, &WebGl2RenderingContext, bool)
            + 'static,
    ) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(
        &mut self,
        point: Vec2<f32>,
        sys: &mut shogo::dots::ShaderSystem,
        ctx: &WebGl2RenderingContext,
        check_naive: bool,
    ) {
        self.0(point, sys, ctx, check_naive);
    }
}

use crate::*;
pub struct DemoIter(usize);

impl DemoIter {
    pub fn new() -> DemoIter {
        DemoIter(0)
    }
    pub fn next(&mut self, area: Vec2<u32>, ctx: &web_sys::WebGl2RenderingContext) -> Demo {
        let curr = self.0;
        //let k=ctx.shader_system();

        let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);

        let k: Demo = match curr {
            0 => liquid::make_demo(area, ctx),
            1 => raycast::make_demo(area, ctx),
            2 => nbody::make_demo(area,ctx),
            /*
            1 => demo_original_order::make_demo(area),
            2 => demo_raycast_f32::make_demo(area, canvas),
            3 => demo_raycast_f32_debug::make_demo(area, canvas),
            4 => demo_multirect::make_demo(area, canvas),
            5 => demo_intersect_with::make_demo(area, canvas),
            6 => demo_knearest::make_demo(area, canvas),
            7 => demo_nbody::make_demo(area),
            8 => demo_raycast_grid::make_demo(area, canvas),
            */
            _ => unreachable!(),
        };
        self.0 += 1;

        if self.0 == 3 {
            self.0 = 0
        }
        k
    }
}
