mod app;

use app::App;

use anyhow::Result;
use std::path::Path;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const OPEN_RUNTIME_MODE: bool = false;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

#[rustfmt::skip]
fn main() -> Result<()> {
    if OPEN_RUNTIME_MODE {
        pretty_env_logger::init();    

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Fate Engine v1.0.0 <Vulkan>")
            .with_inner_size(LogicalSize::new(WIDTH as u32, HEIGHT as u32))
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
        let mut bytes: Vec<u8> = Vec::with_capacity(WIDTH * HEIGHT * 4);
    
        for j in (0..HEIGHT).rev() {
            for i in 0..WIDTH {
                let r = (i as f64) / ((WIDTH - 1) as f64);
                let g = (j as f64) / ((HEIGHT - 1) as f64);
                let b = 0.25;
    
                let ir = (255.999 * r) as u64;
                let ig = (255.999 * g) as u64;
                let ib = (255.999 * b) as u64;
    
                bytes.push(ir as u8);
                bytes.push(ig as u8);
                bytes.push(ib as u8);
            }
        }
        
        image::save_buffer(&Path::new("output.jpeg"), &bytes, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8)?;
        println!("渲染完成");
        return Ok(());
    }
}
