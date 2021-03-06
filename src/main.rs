extern crate alga;
extern crate fps_counter;
extern crate generic_array;
#[macro_use]
extern crate imgui;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics3d as nphysics;
extern crate pathfinding;
extern crate png;
extern crate rand;
extern crate ron;
#[macro_use]
extern crate serde_derive;
extern crate specs;
extern crate typenum;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate wavefront_obj;
extern crate winit;
extern crate app_dirs2;
extern crate rodio;
extern crate show_message;
extern crate palette;
extern crate locale_config;
extern crate fluent_locale;

#[macro_use]
mod util;
mod graphics;
mod audio;
mod entity;
mod component;
mod system;
mod resource;
pub mod maze;
mod config;
mod level;

pub use config::CONFIG;

use vulkano_win::VkSurfaceBuild;
use show_message::UnwrapOrShow;

use vulkano::swapchain;
use vulkano::sync::now;
use vulkano::sync::FenceSignalFuture;
use vulkano::sync::GpuFuture;
use vulkano::instance::Instance;

use winit::{DeviceEvent, Event, WindowEvent, Icon};

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use std::fs::File;
use std::io::Read;


fn init_imgui() -> ::imgui::ImGui {
    let mut imgui = ::imgui::ImGui::init();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    imgui.set_font_global_scale(::CONFIG.font_global_scale);
    imgui.set_mouse_draw_cursor(false);
    imgui.set_imgui_key(::imgui::ImGuiKey::Tab, 0);
    imgui.set_imgui_key(::imgui::ImGuiKey::LeftArrow, 1);
    imgui.set_imgui_key(::imgui::ImGuiKey::RightArrow, 2);
    imgui.set_imgui_key(::imgui::ImGuiKey::UpArrow, 3);
    imgui.set_imgui_key(::imgui::ImGuiKey::DownArrow, 4);
    imgui.set_imgui_key(::imgui::ImGuiKey::PageUp, 5);
    imgui.set_imgui_key(::imgui::ImGuiKey::PageDown, 6);
    imgui.set_imgui_key(::imgui::ImGuiKey::Home, 7);
    imgui.set_imgui_key(::imgui::ImGuiKey::End, 8);
    imgui.set_imgui_key(::imgui::ImGuiKey::Delete, 9);
    imgui.set_imgui_key(::imgui::ImGuiKey::Backspace, 10);
    imgui.set_imgui_key(::imgui::ImGuiKey::Enter, 11);
    imgui.set_imgui_key(::imgui::ImGuiKey::Escape, 12);
    imgui.set_imgui_key(::imgui::ImGuiKey::A, 13);
    imgui.set_imgui_key(::imgui::ImGuiKey::C, 14);
    imgui.set_imgui_key(::imgui::ImGuiKey::V, 15);
    imgui.set_imgui_key(::imgui::ImGuiKey::X, 16);
    imgui.set_imgui_key(::imgui::ImGuiKey::Y, 17);
    imgui.set_imgui_key(::imgui::ImGuiKey::Z, 18);
    CONFIG.style.set_style(imgui.style_mut());
    imgui
}

fn main() {
    // On windows stack is overflowed otherwise
    ::std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
        while let ControlFlow::Restart = new_game() {}
    }).unwrap().join().unwrap();
}

enum ControlFlow {
    Restart,
    Quit,
}

fn new_game() -> ControlFlow {
    ::std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    let mut save = ::resource::Save::new();

    let text = ::resource::Text::load();

    let instance = {
        let extensions = vulkano_win::required_extensions();
        let info = app_info_from_cargo_toml!();
        Instance::new(Some(&info), &extensions, None)
            .unwrap_or_else_show(|e| format!("{}\n\n{}", text.vulkan_error, e))
    };

    let mut events_loop = winit::EventsLoop::new();
    let mut window_builder = winit::WindowBuilder::new();

    if save.fullscreen() {
        window_builder = window_builder.with_fullscreen(Some(events_loop.get_primary_monitor()));
    }

    let icon = {
        let icon_data = if cfg!(feature = "packed") {
            include_bytes!("../assets/icon.png").iter().cloned().collect::<Vec<_>>()
        } else {
            let mut data = vec![];
            let mut file = File::open("assets/icon.png")
                .unwrap_or_else_show(|e| format!("Failed to open \"assets/icon.png\": {}", e));
            file.read_to_end(&mut data)
                .unwrap_or_else_show(|e| format!("Failed to read \"assets/icon.png\": {}", e));
            data
        };
        Icon::from_bytes(&icon_data)
            .unwrap_or_else_show(|e| format!("Failed to load icon: {}", e))
    };

    let window = window_builder
        .with_window_icon(Some(icon))
        .with_title("HyperZen Training")
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap_or_else_show(|e| format!("Failed to build vulkan window: {}\n\n{:#?}", e, e));

    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let current_window_id = window.window().id();

    let mut imgui = init_imgui();
    let mut graphics = graphics::Graphics::new(&window, &mut imgui, &mut save);

    let mut previous_frame_end: Option<FenceSignalFuture<Box<GpuFuture>>> = None;

    let debug = ::std::env::var("HYPERZEN_TRAINING_DEBUG").map(|v| v == "1").unwrap_or(false);

    let mut world = specs::World::new();
    world.register::<::component::Player>();
    world.register::<::component::Teleport>();
    world.register::<::component::Generator>();
    world.register::<::component::Shooter>();
    world.register::<::component::Hook>();
    world.register::<::component::WeaponAnimation>();
    world.register::<::component::Aim>();
    world.register::<::component::StaticDraw>();
    world.register::<::component::DynamicDraw>();
    world.register::<::component::DynamicEraser>();
    world.register::<::component::DynamicHud>();
    world.register::<::component::DynamicGraphicsAssets>();
    world.register::<::component::DeletBool>();
    world.register::<::component::DeletTimer>();
    world.register::<::component::Reducer>();
    world.register::<::component::PhysicBody>();
    world.register::<::component::Activated>();
    world.register::<::component::Momentum>();
    world.register::<::component::Avoider>();
    world.register::<::component::Bouncer>();
    world.register::<::component::Turret>();
    world.register::<::component::DepthBall>();
    world.register::<::component::Attracted>();
    world.register::<::component::Motionless>();
    world.register::<::component::Life>();
    world.register::<::component::Contactor>();
    world.register::<::component::Proximitor>();
    world.register::<::component::FollowPlayer>();
    world.register::<::component::PhysicSensor>();
    world.add_resource(::resource::Help(String::new()));
    world.add_resource(text);
    world.add_resource(graphics.clone());
    world.add_resource(Some(imgui));
    world.add_resource(::resource::Events(vec![]));
    world.add_resource(instance.clone());
    world.add_resource(::resource::Rendering::new());
    world.add_resource(::resource::DebugMode(debug));
    world.add_resource(::resource::FpsCounter(0));
    world.add_resource(::resource::PlayerControl::new());
    world.add_resource(::resource::Benchmarks::new());
    world.add_resource(::resource::UpdateTime(0.0));
    world.add_resource(::resource::GameDuration(Duration::new(0, 0)));
    world.add_resource(::resource::Activated(false));
    world.add_resource(::resource::Audio::init(&save));
    world.add_resource(::resource::LevelActions(vec![]));
    world.add_resource(::resource::ErasedStatus::new());
    let menu_state = ::resource::MenuState::new(&save);
    world.add_resource(save);
    world.add_resource(menu_state);
    world.maintain();

    let mut game_system = ::system::GameSystem::new();
    game_system.run(&mut world);

    let mut pause_update_dispatcher = ::specs::DispatcherBuilder::new()
        .add(::system::MenuPauseControlSystem::new(), "menu_pause", &[])
        .add(::system::HelpSystem, "help", &[])
        .add(::system::AudioSystem, "audio", &[])
        .build();

    let mut game_update_dispatcher = ::specs::DispatcherBuilder::new()
        .add(::system::AudioSystem, "audio", &[])
        .add(::system::MenuGameControlSystem, "menu_game", &[])
        .add(::system::PlayerControlSystem, "player_control", &[])
        .add(::system::AvoiderControlSystem, "avoider_control", &[])
        .add(::system::BouncerControlSystem, "bouncer_control", &[])
        .add(::system::TeleportSystem, "teleport", &[])
        .add(::system::FollowPlayerSystem, "follower_control", &[])
        .add(::system::TurretControlSystem::new(), "turret_control", &[])
        .add(::system::GeneratorSystem, "generator", &[])
        .add(::system::ShootSystem::new(), "shoot", &[])
        .add(::system::HookSystem::new(), "hook", &[])
        .add(::system::PhysicSystem, "physic", &[])
        .add(::system::DeleterSystem, "deleter", &[])
        .add(::system::PlayerDeathSystem, "death", &[])
        .add(::system::ActivateSystem, "activate", &[])
        .add(::system::ReducerSystem, "reducer", &[])
        .add(::system::DepthCoefSystem, "depth_coef", &[])
        .add(::system::DepthBallSystem, "depth_ball", &[])
        .add(::system::AttractedSystem::new(), "attracted", &[])
        .add_barrier() // following systems will delete physic bodies
        .add(::system::LifeSystem, "life", &[])
        .build();

    let mut prepare_game_draw_dispatcher = ::specs::DispatcherBuilder::new()
        .add(
            ::system::UpdateDynamicDrawEraserSystem,
            "update_dynamic_draw",
            &[],
        )
        .build();

    let mut draw_dispatcher = ::specs::DispatcherBuilder::new()
        .add(::system::DrawSystem, "draw_system", &[])
        .build();

    {
        assert!(world.read_resource::<::resource::UpdateTime>().0 == 0.0);
        game_update_dispatcher.dispatch(&mut world.res);
        world.maintain();
        prepare_game_draw_dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    let frame_duration = Duration::new(
        0,
        (1_000_000_000.0 / ::CONFIG.fps as f32) as u32,
    );
    let mut last_frame_instant = Instant::now();
    let mut fps_counter = fps_counter::FPSCounter::new();
    let mut benchmarker = util::Benchmarker::new();
    let mut last_update_instant = Instant::now();

    loop {
        benchmarker.start("pre_update");
        if let Some(ref mut previous_frame_end) = previous_frame_end {
            previous_frame_end.cleanup_finished();
        }

        // Poll events
        {
            let mut events = world.write_resource::<::resource::Events>();

            events.0.clear();

            let mut done = false;

            events_loop.poll_events(|ev| {
                // Filter out old window event
                match ev {
                    Event::WindowEvent {
                        window_id,
                        ..
                    } => if window_id != current_window_id {
                        return
                    }
                    _ => (),
                }

                let retain = match ev {
                    Event::WindowEvent {
                        event: WindowEvent::Focused(true),
                        ..
                    } => {
                        try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Normal), 100, 10)
                            .unwrap_or_else_show(|e| format!("Failed to reset cursor: {}", e));
                        try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab), 100, 10)
                            .unwrap_or_else_show(|e| format!("Failed to grab cursor: {}", e));
                        false
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        done = true;
                        false
                    }
                    Event::WindowEvent {
                        event: WindowEvent::MouseInput { .. },
                        ..
                    }
                    | Event::WindowEvent {
                        event: WindowEvent::ReceivedCharacter(..),
                        ..
                    }
                    | Event::WindowEvent {
                        event: WindowEvent::MouseWheel { .. },
                        ..
                    }
                    | Event::WindowEvent {
                        event: WindowEvent::KeyboardInput { .. },
                        ..
                    }
                    | Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { .. },
                        ..
                    }
                    | Event::WindowEvent {
                        event: WindowEvent::CursorMoved { .. },
                        ..
                    } => true,
                    _ => false,
                };

                if retain {
                    events.0.push(ev);
                }
            });
            if done {
                break ControlFlow::Quit;
            }
        }

        if world.write_resource::<::resource::MenuState>().quit_button {
            break ControlFlow::Quit;
        }
        if world.write_resource::<::resource::MenuState>().restart_now_button {
            break ControlFlow::Restart;
        }
        benchmarker.end("pre_update");

        // Update world
        benchmarker.start("update");

        let delta_time = last_update_instant.elapsed();
        last_update_instant = Instant::now();

        if world.read_resource::<::resource::MenuState>().paused() {
            world.write_resource::<::resource::UpdateTime>().0 = 0.0;
            pause_update_dispatcher.dispatch(&mut world.res);
        } else {
            world.write_resource::<::resource::GameDuration>().0 += delta_time;
            world.write_resource::<::resource::UpdateTime>().0 = delta_time
                .as_secs()
                .saturating_mul(1_000_000_000)
                .saturating_add(delta_time.subsec_nanos() as u64)
                as f32 / 1_000_000_000.0;
            game_update_dispatcher.dispatch(&mut world.res);
            world.maintain();
            game_system.run(&mut world);
            prepare_game_draw_dispatcher.dispatch(&mut world.res);
            world.maintain();
        }

        benchmarker.end("update");

        // Render world
        benchmarker.start("draw");

        // On X with Xmonad and intel HD graphics the acquire stay sometimes forever
        let timeout = Duration::from_secs(2);
        let mut next_image = swapchain::acquire_next_image(graphics.swapchain.clone(), Some(timeout));
        loop {
            match next_image {
                Err(vulkano::swapchain::AcquireError::OutOfDate)
                | Err(vulkano::swapchain::AcquireError::Timeout) => {
                    // Drop ImGui
                    *world.write_resource::<::resource::ImGuiOption>() = None;

                    let mut imgui = init_imgui();
                    graphics.recreate(&window, &mut imgui);
                    *world.write_resource::<::resource::ImGuiOption>() = Some(imgui);
                    *world.write_resource() = graphics.clone();
                    next_image = swapchain::acquire_next_image(graphics.swapchain.clone(), Some(timeout));
                }
                _ => break
            }
        }

        let (image_num, acquire_future) = next_image
            .unwrap_or_else_show(|e| format!("Failed to acquire next image: {}", e));

        world.write_resource::<::resource::Rendering>().image_num = Some(image_num);
        world.write_resource::<::resource::Rendering>().size = window.window().get_inner_size();

        draw_dispatcher.dispatch(&mut world.res);

        let (command_buffer, second_command_buffer) = {
            let mut rendering = world.write_resource::<::resource::Rendering>();
            (
                rendering.command_buffer.take().unwrap(),
                rendering.second_command_buffer.take().unwrap(),
            )
        };

        if let Some(previous_frame_end) = previous_frame_end {
            if let Ok(()) = previous_frame_end.wait(None) {
                let amount = graphics.erased_amount_buffer.read().unwrap()[0];
                let total = graphics.dim[0] as f32 * graphics.dim[1] as f32;
                world.write_resource::<::resource::ErasedStatus>().amount = amount as f32 / total;
            }
        }

        let future = now(graphics.device.clone())
            .then_execute(graphics.queue.clone(), command_buffer)
            .unwrap()
            .join(acquire_future)
            .then_execute(graphics.queue.clone(), second_command_buffer)
            .unwrap()
            .then_swapchain_present(
                graphics.queue.clone(),
                graphics.swapchain.clone(),
                image_num,
            );

        let future = Box::new(future) as Box<GpuFuture>;
        let future = future.then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Some(future);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                previous_frame_end = None;
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
                previous_frame_end = None;
            }
        }
        benchmarker.end("draw");

        // Sleep
        benchmarker.start("sleep");
        let elapsed = last_frame_instant.elapsed();
        if let Some(to_sleep) = frame_duration.checked_sub(elapsed) {
            thread::sleep(to_sleep);
        }
        last_frame_instant = Instant::now();
        world
            .write_resource::<::resource::FpsCounter>()
            .0 = fps_counter.tick();
        benchmarker.end("sleep");
        *world.write_resource::<::resource::Benchmarks>() = benchmarker.get_all();
    }
}
