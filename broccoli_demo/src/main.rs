use axgeom::vec2;
use axgeom::Vec2;
use axgeom::*;
use egaku2d::glutin;
use glutin::event::ElementState;
use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;

#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
use duckduckgeo::F32n;

use self::support::prelude::*;

pub struct Demo(Box<dyn FnMut(Vec2<F32n>, &mut SimpleCanvas, bool)>);
impl Demo {
    pub fn new(func: impl FnMut(Vec2<F32n>, &mut SimpleCanvas, bool) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(&mut self, point: Vec2<F32n>, sys: &mut SimpleCanvas, check_naive: bool) {
        self.0(point, sys, check_naive);
    }
}

mod demo_iter {
    use crate::demos::*;
    use crate::*;
    pub struct DemoIter(usize);

    impl DemoIter {
        pub fn new() -> DemoIter {
            DemoIter(0)
        }
        pub fn next(&mut self, area: Vec2<u32>, canvas: &mut SimpleCanvas) -> Demo {
            let curr = self.0;

            let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);
            let area: Rect<F32n> = area.inner_try_into().unwrap();

            let k: Demo = match curr {
                0 => demo_liquid::make_demo(area),
                1 => demo_raycast_f32::make_demo(area, canvas),
                2 => demo_raycast_f32_debug::make_demo(area, canvas),
                3 => demo_multirect::make_demo(area, canvas),
                4 => demo_original_order::make_demo(area),
                5 => demo_intersect_with::make_demo(area, canvas),
                6 => demo_knearest::make_demo(area, canvas),
                7 => demo_nbody::make_demo(area),
                8 => demo_raycast_grid::make_demo(area, canvas),

                _ => unreachable!("Not possible"),
            };
            self.0 += 1;

            if self.0 == 9 {
                self.0 = 0
            }
            k
        }
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global()
        .unwrap();

    let area = vec2(800, 600);

    let events_loop = glutin::event_loop::EventLoop::new();

    let mut sys = egaku2d::WindowedSystem::new([800, 600], &events_loop, "dinotree_alg demo");
    //let mut sys=very_simple_2d::FullScreenSystem::new(&events_loop);
    //sys.set_viewport_min(600.);

    let mut demo_iter = demo_iter::DemoIter::new();

    let mut curr = demo_iter.next(area, sys.canvas_mut());

    println!("Press \"N\" to go to the next example");

    let mut check_naive = false;
    let mut cursor = vec2same(0.);
    let mut timer = egaku2d::RefreshTimer::new(16);
    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                *control_flow = ControlFlow::Exit;
                            }
                            Some(VirtualKeyCode::N) => {
                                curr = demo_iter.next(area, sys.canvas_mut());
                            }
                            Some(VirtualKeyCode::C) => {
                                check_naive = !check_naive;
                                if check_naive {
                                    println!("naive checking is on");
                                } else {
                                    println!("naive checking is off");
                                }
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_logical_size) => {}
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                } => {
                    //let dpi=sys.get_hidpi_factor();
                    //let glutin::dpi::PhysicalPosition { x, y } = logical_position.to_physical(dpi);
                    cursor = vec2(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    ..
                } => {
                    if button == glutin::event::MouseButton::Left {
                        match state {
                            glutin::event::ElementState::Pressed => {
                                //mouse_active=true;
                            }
                            glutin::event::ElementState::Released => {
                                //mouse_active=false;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if timer.is_ready() {
                    let k = sys.canvas_mut();
                    k.clear_color([0.2, 0.2, 0.2]);
                    curr.step(cursor.inner_try_into().unwrap(), k, check_naive);
                    sys.swap_buffers();
                }
            }
            _ => {}
        }
    });
}
