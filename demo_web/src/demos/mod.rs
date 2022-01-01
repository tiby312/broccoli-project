pub mod liquid;


use crate::*;
pub struct DemoIter(usize);

impl DemoIter {
    pub fn new() -> DemoIter {
        DemoIter(0)
    }
    pub fn next(&mut self, area: Vec2<u32>,ctx:&web_sys::WebGl2RenderingContext) -> Demo {
        let curr = self.0;
        //let k=ctx.shader_system();
    
        let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);

        let k: Demo = match curr {
            0 => liquid::make_demo(area,ctx),
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

        if self.0 == 9 {
            self.0 = 0
        }
        k
    }
}