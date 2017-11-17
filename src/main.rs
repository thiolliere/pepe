extern crate winit;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate fps_clock;
extern crate alga;
extern crate specs;
extern crate nalgebra as na;
extern crate ncollide;
extern crate rand;
extern crate nphysics3d as nphysics;
#[macro_use]
extern crate lazy_static;
extern crate pathfinding;
extern crate png;
#[macro_use]
extern crate imgui;
#[macro_use]
extern crate serde_derive;
extern crate ron;
extern crate wavefront_obj;

mod util;
mod graphics;
mod entity;
mod component;
mod system;
mod resource;
mod maze;
mod config;
mod testing;

use vulkano_win::VkSurfaceBuild;

use vulkano::swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::instance::Instance;

use winit::{WindowEvent, Event, ElementState, VirtualKeyCode, KeyboardInput, DeviceEvent};

use std::sync::Arc;

pub use testing::TS;

fn main() {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        let info = app_info_from_cargo_toml!();
        Instance::new(Some(&info), &extensions, None).expect("failed to create Vulkan instance")
    };

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let mut error_timer = 0;
    while let Err(_) = window.window().set_cursor_state(winit::CursorState::Grab) {
        ::std::thread::sleep(::std::time::Duration::from_millis(1));
        error_timer += 1;
        if error_timer > 100 {
            panic!("cursor could not be grabbed");
        }
    }

    let config = ::resource::Config::load();

    let mut imgui = ::imgui::ImGui::init();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
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

    config.style.set_style(imgui.style_mut());

    let mut graphics = graphics::Graphics::new(&window, &mut imgui);

    let mut previous_frame_end = Box::new(now(graphics.data.device.clone())) as Box<GpuFuture>;

    let mut maze = ::maze::kruskal(51, 51, 20.0);
    maze.reduce(2);
    maze.circle();
    maze.fill_smallest();
    maze.fill_dead_end();

    let mut world = specs::World::new();
    world.register::<::component::Player>();
    world.register::<::component::Shooter>();
    world.register::<::component::WeaponAnimation>();
    world.register::<::component::WeaponAnchor>();
    world.register::<::component::Aim>();
    world.register::<::component::StaticDraw>();
    world.register::<::component::DynamicDraw>();
    world.register::<::component::DynamicEraser>();
    world.register::<::component::DynamicGraphicsAssets>();
    world.register::<::component::Deleter>();
    world.register::<::component::PhysicBody>();
    world.register::<::component::Momentum>();
    world.register::<::component::Avoider>();
    world.register::<::component::Bouncer>();
    world.register::<::component::Life>();
    world.register::<::component::Contactor>();
    world.add_resource(graphics.data.clone());
    world.add_resource(imgui);
    world.add_resource(config);
    world.add_resource(::resource::PhysicWorld::new());
    world.add_resource(::resource::Rendering::new());
    world.add_resource(::resource::GameEvents(vec![]));
    world.add_resource(::resource::MenuEvents(vec![]));
    world.add_resource(maze);
    world.add_resource(::resource::DebugMode(false));

    {
        ::entity::create_maze_walls(
            &mut world.write(),
            &mut world.write(),
            &mut world.write_resource(),
            &world.read_resource(),
            &world.read_resource(),
            &world.read_resource(),
        );
        // ::entity::create_avoider(
        //     world
        //         .read_resource::<::resource::Maze>()
        //         .random_free_float(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write_resource(),
        //     &world.read_resource());
        // ::entity::create_bouncer(
        //     world
        //         .read_resource::<::resource::Maze>()
        //         .random_free_float(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write(),
        //     &mut world.write_resource(),
        //     &world.read_resource());
        ::entity::create_player(
            world
                .read_resource::<::resource::Maze>()
                .random_free_float(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write(),
            &mut world.write_resource(),
            &world.read_resource(),
        );
    }

    world.maintain();

    let mut update_dispatcher = ::specs::DispatcherBuilder::new()
        .add(::system::MenuControlSystem::new(), "menu", &[])
        .add(::system::PlayerControlSystem::new(), "player_control", &[])
        .add(::system::AvoiderControlSystem, "avoider_control", &[])
        .add(::system::BouncerControlSystem, "bouncer_control", &[])
        .add(::system::ShootSystem::new(), "shoot", &[])
        .add(::system::LifeSystem, "life", &[])
        .add(::system::MazeMasterSystem, "maze_master", &[])
        .add(::system::PhysicSystem, "physic", &[])
        .add(::system::DeleterSystem, "deleter", &[])
        .build();

    let mut draw_dispatcher = ::specs::DispatcherBuilder::new()
        .add(
            ::system::UpdateDynamicDrawEraserSystem,
            "update_dynamic_draw",
            &[],
        )
        .add(::system::DrawSystem, "draw_system", &[])
        .build();

    let mut fps = fps_clock::FpsClock::new(world.read_resource::<::resource::Config>().fps.clone());

    loop {
        previous_frame_end.cleanup_finished();

        // Poll events
        {
            let mut menu_events = world.write_resource::<::resource::MenuEvents>();
            let mut game_events = world.write_resource::<::resource::GameEvents>();
            let mut debug_mode = world.write_resource::<::resource::DebugMode>();

            menu_events.0.clear();
            game_events.0.clear();

            let mut done = false;

            events_loop.poll_events(|ev| {
                let retain = match ev {
                    Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                        done = true;
                        false
                    }
                    Event::WindowEvent { event: WindowEvent::MouseMoved { .. }, .. } => {
                        if !debug_mode.0 {
                            window
                                .window()
                                .set_cursor_position(
                                    graphics.data.dim[0] as i32 / 2,
                                    graphics.data.dim[1] as i32 / 2,
                                )
                                .unwrap();
                        }
                        true
                    }
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::P),
                                ..
                            },
                            ..
                        },
                        ..
                    } => {
                        debug_mode.0 = !debug_mode.0;
                        world
                            .write_resource::<::resource::ImGui>()
                            .set_mouse_draw_cursor(debug_mode.0);
                        true
                    }
                    Event::WindowEvent { event: WindowEvent::MouseInput { .. }, .. } |
                    Event::WindowEvent { event: WindowEvent::ReceivedCharacter(..), .. } |
                    Event::WindowEvent { event: WindowEvent::MouseWheel { .. }, .. } |
                    Event::WindowEvent { event: WindowEvent::KeyboardInput { .. }, .. } |
                    Event::DeviceEvent { event: DeviceEvent::Motion { .. }, .. } |
                    Event::WindowEvent { event: WindowEvent::AxisMotion { .. }, .. } => true,
                    _ => false,
                };

                if retain {
                    if debug_mode.0 {
                        menu_events.0.push(ev);
                    } else {
                        game_events.0.push(ev);
                    }
                }
            });
            if done {
                return;
            }
        }

        update_dispatcher.dispatch(&mut world.res);

        world.maintain();

        // Render world
        let mut next_image = swapchain::acquire_next_image(graphics.data.swapchain.clone(), None);
        while let Err(vulkano::swapchain::AcquireError::OutOfDate) = next_image {
            graphics.recreate(&window);
            *world.write_resource() = graphics.data.clone();
            next_image = swapchain::acquire_next_image(graphics.data.swapchain.clone(), None);
        }
        let (image_num, acquire_future) = next_image.unwrap();

        world.write_resource::<::resource::Rendering>().image_num = Some(image_num);
        world.write_resource::<::resource::Rendering>().size_points =
            window.window().get_inner_size_points();
        world.write_resource::<::resource::Rendering>().size_pixels =
            window.window().get_inner_size_pixels();

        draw_dispatcher.dispatch(&mut world.res);

        let (command_buffer, second_command_buffer) = {
            let mut rendering = world.write_resource::<::resource::Rendering>();
            (
                rendering.command_buffer.take().unwrap(),
                rendering.second_command_buffer.take().unwrap(),
            )
        };

        let future = previous_frame_end
            .then_execute(graphics.data.queue.clone(), command_buffer)
            .unwrap()
            .join(acquire_future)
            .then_execute(graphics.data.queue.clone(), second_command_buffer)
            .unwrap()
            .then_swapchain_present(
                graphics.data.queue.clone(),
                graphics.data.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush()
            .unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

        // Sleep
        fps.tick();
    }
}
