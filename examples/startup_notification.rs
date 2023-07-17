//! Demonstrates the use of startup notifications on Linux.

fn main() {
    example::main();
}

#[cfg(any(x11_platform, wayland_platform))]
#[path = "./util/fill.rs"]
mod fill;

#[cfg(any(x11_platform, wayland_platform))]
mod example {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use winit::event::{ElementState, Event, KeyEvent, StartCause, WindowEvent};
    use winit::event_loop::EventLoop;
    use winit::keyboard::Key;
    use winit::platform::startup_notify::{
        EventLoopExtStartupNotify, WindowBuilderExtStartupNotify, WindowExtStartupNotify,
    };
    use winit::window::{Window, WindowBuilder, WindowId};

    pub(super) fn main() {
        // Create the event loop and get the activation token.
        let event_loop = EventLoop::new();
        let mut current_token = match event_loop.read_token_from_env() {
            Some(token) => Some(token),
            None => {
                println!("No startup notification token found in environment.");
                None
            }
        };

        let mut windows: HashMap<WindowId, Rc<Window>> = HashMap::new();
        let mut counter = 0;
        let mut window_deadline = Some(Instant::now() + Duration::from_secs(2));
        let mut last_window_created = Instant::now();

        event_loop.run(move |event, elwt, flow| {
            match event {
                Event::NewEvents(StartCause::Init) => {
                    println!("Waiting one second to make the notification more obvious...");
                }

                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key,
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                } => {
                    let diff_time = Instant::now().saturating_duration_since(last_window_created);
                    if logical_key == Key::Character("n".into())
                        && diff_time > Duration::from_millis(250)
                    {
                        if let Some(window) = windows.get(&window_id) {
                            // Request a new activation token on this window.
                            // Once we get it we will use it to create a window.
                            last_window_created = Instant::now();
                            window
                                .request_activation_token()
                                .expect("Failed to request activation token.");
                        }
                    }
                }

                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                } => {
                    // Remove the window from the map.
                    windows.remove(&window_id);
                    if windows.is_empty() {
                        flow.set_exit();
                        return;
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::ActivationTokenDone { token, .. },
                    ..
                } => {
                    window_deadline = Some(Instant::now() + Duration::from_secs(2));
                    current_token = Some(token);
                }

                Event::RedrawRequested(id) => {
                    if let Some(window) = windows.get(&id) {
                        super::fill::fill_window(window);
                    }
                }

                _ => {}
            }

            // See if we've passed the deadline.
            if window_deadline.map_or(false, |d| d <= Instant::now()) {
                // Create the initial window.
                let window = {
                    let mut builder =
                        WindowBuilder::new().with_title(format!("Window {}", counter));

                    if let Some(current_token) = current_token.take() {
                        builder = builder.with_activation_token(current_token);
                    }

                    Rc::new(builder.build(elwt).unwrap())
                };

                // Add the window to the map.
                windows.insert(window.id(), window.clone());

                counter += 1;
                window_deadline = None;
            }

            match window_deadline {
                Some(deadline) => flow.set_wait_until(deadline),
                None => flow.set_wait(),
            }
        });
    }
}

#[cfg(not(any(x11_platform, wayland_platform)))]
mod example {
    pub(super) fn main() {
        println!("This example is only supported on X11 and Wayland platforms.");
    }
}
