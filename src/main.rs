mod application;
mod render_pass;
mod user_io;
mod signed_distance_function_renderer;

use std::time::Instant;
use vulkano::image::ImageAccess;
use vulkano_util::window::WindowDescriptor;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use crate::application::{Application};
use crate::user_io::UserIO;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut app = Application::default();

    let main_window_descriptor = WindowDescriptor {
        width: 1024.0,
        height: 1024.0,
        title: "Triangle".to_string(),
        ..Default::default()
    };

    app.open_new_window(&event_loop, main_window_descriptor);

    // Time & inputs...
    let mut time = Instant::now();
    let mut user_input = UserIO::new();
    loop {

        // Window event handling.
        if !handle_events(&mut event_loop, &mut app, &mut user_input) {
            break;
        }

        // Compute life & render 60fps.
        if (Instant::now() - time).as_secs_f64() > 1.0 / 60.0 {
            for (window_id, window_renderer) in app.windows.iter_mut() {
                let pipeline = app.pipelines.get_mut(window_id).unwrap();

                // Skip this window when minimized.
                match window_renderer.window_size() {
                    [w, h] => {
                        if w == 0.0 || h == 0.0 {
                            return;
                        }
                    }
                }

                // Start the frame.
                let before_pipeline_future = match window_renderer.acquire() {
                    Err(e) => {
                        println!("{e}");
                        return;
                    }
                    Ok(future) => future,
                };

                let after_compute = pipeline.compute.compute(before_pipeline_future);
                let color_image = pipeline.compute.color_image();
                let target_image = window_renderer.swapchain_image_view();

                let after_render = pipeline.place_over_frame.render(after_compute, color_image, target_image);
                window_renderer.present(after_render, true);
            }
            time = Instant::now();
        }
    }
}

/// Handles events and returns a `bool` indicating if we should quit.
fn handle_events(
    event_loop: &mut EventLoop<()>,
    app: &mut Application,
    user_input: &mut UserIO
) -> bool {
    let mut is_running = true;
    event_loop.run_return(|window_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match &window_event {
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        user_input.set_mouse_delta(delta.0, delta.1);
                    }
                    _ => {}
                }
            },

            Event::WindowEvent { event, window_id, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        if *window_id == app.windows.primary_window_id().unwrap() {
                            is_running = false;
                        } else {

                            // Destroy window by removing its renderer.
                            app.windows.remove_renderer(*window_id);
                            app.pipelines.remove(window_id);
                        }
                    }

                    // Resize window and its images.
                    WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                        let vulkano_window = app.windows.get_renderer_mut(*window_id).unwrap();
                        vulkano_window.resize();
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        user_input.set_keyboard_input(input);
                    },

                    // Handle mouse button events.
                    WindowEvent::MouseInput { .. } => {}

                    _ => (),
                }
            }

            Event::MainEventsCleared => *control_flow = ControlFlow::Exit,

            _ => (),
        }
    });

    is_running
}
