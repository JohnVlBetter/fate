mod app;

use app::App;

use anyhow::Result;
use fate_rt::renderer::Renderer;
use std::path::Path;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

#[rustfmt::skip]
fn main() -> Result<()> {
    pretty_env_logger::init();    

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Fate Engine v1.0.0 <Vulkan>")
        .with_inner_size(LogicalSize::new(WIDTH as u32, HEIGHT as u32))
        .build(&event_loop)?;

    let mut app = unsafe { App::new(&window) }?;
    let mut destroying = false;
    let mut minimized = false;

    let renderer = Renderer::new()?;

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
                        Some(VirtualKeyCode::Space) => {
                            let _ = renderer.render(WIDTH, HEIGHT, Path::new("output.ppm"));
                        }
                        _ => { }
                    }
                }
            }
            _ => {}
        }
    });
}
