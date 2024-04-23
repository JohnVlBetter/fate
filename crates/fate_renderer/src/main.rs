mod camera;
mod config;
mod gui;
mod inputsystem;
mod loader;
mod renderer;
mod transform;

use crate::{camera::*, config::Config, gui::Gui, inputsystem::*, loader::*, renderer::*};
use log::LevelFilter;
use rendering::cgmath::{Matrix4, Vector3};
use rendering::environment::Environment;
use rendering::{animation::PlaybackMode, model::Model};
use std::{cell::RefCell, error::Error, path::PathBuf, rc::Rc, sync::Arc, time::Instant};
use vulkan::*;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::set_max_level(LevelFilter::Error);
    log::info!("Fate初始化开始...");

    let config = Default::default();
    let enable_debug = true;
    let file_path = Some(PathBuf::from(
        "assets/models/DamagedHelmet/glTF/DamagedHelmet.gltf",
    ));
    run(config, enable_debug, file_path);

    Ok(())
}

fn run(config: Config, enable_debug: bool, path: Option<PathBuf>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = WindowBuilder::new()
        .with_title("Fate")
        .with_inner_size(PhysicalSize::new(
            config.resolution().width(),
            config.resolution().height(),
        ))
        .with_fullscreen(config.fullscreen().then_some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let context = Arc::new(Context::new(&window, enable_debug));

    let renderer_settings = RendererSettings::default();

    let environment = Environment::new(&context, config.env().path(), config.env().resolution());
    let mut gui = Gui::new(&window, renderer_settings);
    let mut renderer = Renderer::create(
        Arc::clone(&context),
        &config,
        renderer_settings,
        environment,
    );

    let mut model: Option<Rc<RefCell<Model>>> = None;
    let loader = Loader::new(Arc::new(context.new_thread()));
    if let Some(p) = path {
        loader.load(p);
    }

    let mut camera = Camera::default();
    let mut input_state = InputSystem::default();
    let mut time = Instant::now();
    let mut dirty_swapchain = false;

    log::debug!("Fate初始化完毕");
    event_loop
        .run(move |event, elwt| {
            input_state = input_state.update(&event);

            match event {
                Event::NewEvents(_) => {}
                Event::AboutToWait => {
                    let new_time = Instant::now();
                    let delta_s = (new_time - time).as_secs_f64();
                    time = new_time;

                    if let Some(loaded_model) = loader.get_model() {
                        gui.set_model_metadata(loaded_model.metadata().clone());
                        model.take();

                        context.graphics_queue_wait_idle();
                        let loaded_model = Rc::new(RefCell::new(loaded_model));
                        renderer.set_model(&loaded_model);
                        model = Some(loaded_model);
                    }

                    if let Some(model) = model.as_ref() {
                        let mut model = model.borrow_mut();

                        if input_state.is_control_w_clicked() {
                            println!("eeee");
                            model.transform(Some(Matrix4::from_translation(Vector3::new(0.0,0.0,0.01))));
                        }

                        if gui.should_toggle_animation() {
                            model.toggle_animation();
                        } else if gui.should_stop_animation() {
                            model.stop_animation();
                        } else if gui.should_reset_animation() {
                            model.reset_animation();
                        } else {
                            let playback_mode = if gui.is_infinite_animation_checked() {
                                PlaybackMode::Loop
                            } else {
                                PlaybackMode::Once
                            };

                            model.set_animation_playback_mode(playback_mode);
                            model.set_current_animation(gui.get_selected_animation());
                        }
                        gui.set_animation_playback_state(model.get_animation_playback_state());

                        let delta_s = delta_s as f32 * gui.get_animation_speed();
                        model.update(delta_s);
                    }

                    {
                        if gui.should_reset_camera() {
                            camera = Default::default();
                        }

                        if !gui.is_hovered() {
                            camera.update(&input_state);
                            gui.set_camera(Some(camera));
                        }
                    }

                    if let Some(renderer_settings) = gui.get_new_renderer_settings() {
                        renderer.update_settings(renderer_settings);
                    }

                    if dirty_swapchain {
                        let PhysicalSize { width, height } = window.inner_size();
                        if width > 0 && height > 0 {
                            renderer.recreate_swapchain(window.inner_size().into(), config.vsync());
                        } else {
                            return;
                        }
                    }

                    dirty_swapchain = matches!(
                        renderer.render(&window, camera, &mut gui),
                        Err(RenderError::DirtySwapchain)
                    );
                }

                Event::WindowEvent { event, .. } => {
                    gui.handle_event(&window, &event);
                    match event {
                        WindowEvent::DroppedFile(path) => {
                            log::debug!("已拖入文件{:?}", path);
                            loader.load(path);
                        }

                        WindowEvent::Resized(new_size) => {
                            log::debug!("窗口尺寸变更为{:?}", new_size);
                            dirty_swapchain = true;
                        }

                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        _ => (),
                    }
                }
                Event::LoopExiting => {
                    log::info!("退出Fate");
                    renderer.wait_idle_gpu();
                }
                _ => (),
            }
        })
        .unwrap();
}
