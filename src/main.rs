mod app;

use app::App;

use anyhow::Result;
use fate_graphic::input_system::InputState;
use fate_rt::renderer::Renderer;
use std::path::Path;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

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

    let mut input_state = InputState::default();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input_state = input_state.update(&event);

        match event {
            Event::MainEventsCleared if !destroying && !minimized => unsafe { app.render(&window, &input_state) }.unwrap(),
            
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
