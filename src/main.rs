mod app;

use app::App;

use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use image::*;

const OPEN_RUNTIME_MODE: bool = false;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

#[rustfmt::skip]
fn main() -> Result<()> {
    if OPEN_RUNTIME_MODE {
        pretty_env_logger::init();    

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Fate Engine v1.0.0 <Vulkan>")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop)?;
    
        let mut app = unsafe { App::new(&window) }?;
        let mut destroying = false;
        let mut minimized = false;
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared if !destroying && !minimized => unsafe { app.render(&window) }.unwrap(),
                
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    if size.width == 0 || size.height == 0 {
                        minimized = true;
                    } else {
                        minimized = false;
                        app.resized = true;
                    }
                }
    
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    destroying = true;
                    *control_flow = ControlFlow::Exit;
                    unsafe { app.destroy(); }
                }
    
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Left) if app.models > 1 => app.models -= 1,
                            Some(VirtualKeyCode::Right) if app.models < 9 => app.models += 1,
                            _ => { }
                        }
                    }
                }
                _ => {}
            }
        });
    }
    else {
        println!("P3");
        println!("{} {}", WIDTH, HEIGHT);
        println!("255");
    
        for j in (0..HEIGHT).rev() {
            for i in 0..WIDTH {
                let r = (i as f64) / ((WIDTH - 1) as f64);
                let g = (j as f64) / ((HEIGHT - 1) as f64);
                let b = 0.25;
    
                let ir = (255.999 * r) as u64;
                let ig = (255.999 * g) as u64;
                let ib = (255.999 * b) as u64;
    
                println!("{} {} {}", ir, ig, ib);
            }
        }
        return Ok(());
    }
}
