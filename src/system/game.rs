use nphysics::resolution::{AccumulatedImpulseSolver, CorrectionMode};
use std::time::Duration;
use std::collections::HashMap;

pub struct GameSystem {
    current_level: Option<Level>,
}

#[derive(Clone, Copy)]
enum Level {
    Hall,
    Custom,
    Level(usize, usize),
}

impl GameSystem {
    pub fn new() -> Self {
        GameSystem {
            current_level: None,
        }
    }
    pub fn run(&mut self, world: &mut ::specs::World) {
        let action = {
            let mut level_actions = world.write_resource::<::resource::LevelActions>();
            let action = level_actions.0.first().cloned();
            level_actions.0.clear();
            action
        };

        let recreate_level = match (self.current_level, action) {
            (None, _) => Some(Level::Hall),
            (Some(Level::Hall), Some(::resource::LevelAction::Level(level))) => {
                world.write_resource::<::resource::Audio>().play_unspatial(::audio::Sound::Portal);
                if ::CONFIG.levels[level].len() != 0 {
                    Some(Level::Level(level, 0))
                } else {
                    let mut game_duration = world.write_resource::<::resource::GameDuration>();
                    world.write_resource::<::resource::Save>().insert_score(level, Duration::new(0, 0));
                    game_duration.0 = Duration::new(0, 0);
                    Some(Level::Hall)
                }
            },
            (Some(Level::Level(level, part)), Some(::resource::LevelAction::Next)) => {
                world.write_resource::<::resource::Audio>().play_unspatial(::audio::Sound::Portal);
                if ::CONFIG.levels[level].len() > part + 1 {
                    Some(Level::Level(level, part+1))
                } else {
                    let mut game_duration = world.write_resource::<::resource::GameDuration>();
                    world.write_resource::<::resource::Save>().insert_score(level, game_duration.0);
                    game_duration.0 = Duration::new(0, 0);
                    Some(Level::Hall)
                }
            },
            (current_level, Some(::resource::LevelAction::Reset)) => current_level,
            (_, Some(::resource::LevelAction::Custom)) => Some(Level::Custom),
            (_, Some(::resource::LevelAction::ReturnHall)) => Some(Level::Hall),
            (Some(_), None) => None,

            (Some(Level::Hall), Some(::resource::LevelAction::Next)) => {
                println!("INTERNAL ERROR: called next in hall");
                Some(Level::Hall)
            },
            (Some(Level::Custom), Some(::resource::LevelAction::Next)) => {
                println!("INTERNAL ERROR: called next in custom");
                Some(Level::Hall)
            },
            (Some(Level::Level(..)), Some(::resource::LevelAction::Level(..)))
            | (Some(Level::Custom), Some(::resource::LevelAction::Level(..)))
            => {
                println!("INTERNAL ERROR: called go to level outside hall");
                Some(Level::Hall)
            },
        };

        if let Some(level) = recreate_level {
            world.write_resource::<::resource::GameDuration>().0 = Duration::new(0, 0);
            world.write_resource::<::resource::Activated>().0 = false;
            self.current_level = Some(level);

            let physic_world = {
                let mut physic_world = ::resource::PhysicWorld::new();
                *physic_world.constraints_solver() = AccumulatedImpulseSolver::new(
                    ::CONFIG.accumulated_impulse_solver_step,
                    CorrectionMode::VelocityAndPosition(
                        ::CONFIG.correction_mode_a,
                        ::CONFIG.correction_mode_b,
                        ::CONFIG.correction_mode_c,
                    ),
                    ::CONFIG.accumulated_impulse_solver_joint_corr_factor,
                    ::CONFIG.accumulated_impulse_solver_rest_eps,
                    ::CONFIG.accumulated_impulse_solver_num_first_order_iter,
                    ::CONFIG.accumulated_impulse_solver_num_second_order_iter,
                );
                physic_world
            };

            world.delete_all();
            world.add_resource(::resource::Events(vec![]));

            world.write_resource::<::resource::Graphics>().reset_group();
            world.write_resource::<::resource::ErasedStatus>().clear();

            world.add_resource(::resource::DepthCoef(1.0));
            world.add_resource(physic_world);

            match level {
                Level::Hall => ::level::create_hall(world),
                Level::Level(level, part) => ::CONFIG.levels[level][part].create(world),
                Level::Custom => {
                    let conf = world.read_resource::<::resource::Save>().custom_level_conf();

                    let mut entities = HashMap::new();
                    entities.insert(::entity::EntityConf::MotionLess { eraser: false }, conf.motion_less as usize);
                    entities.insert(::entity::EntityConf::MotionLess { eraser: true }, conf.motion_less_eraser as usize);
                    entities.insert(::entity::EntityConf::Attracted { eraser: false }, conf.attracted as usize);
                    entities.insert(::entity::EntityConf::Attracted { eraser: true }, conf.attracted_eraser as usize);
                    entities.insert(::entity::EntityConf::Bouncer { eraser: false }, conf.bouncer as usize);
                    entities.insert(::entity::EntityConf::Bouncer { eraser: true }, conf.bouncer_eraser as usize);
                    entities.insert(::entity::EntityConf::Avoider { eraser: false }, conf.avoider as usize);
                    entities.insert(::entity::EntityConf::Avoider { eraser: true }, conf.avoider_eraser as usize);
                    entities.insert(::entity::EntityConf::Turret, conf.turret as usize);

                    ::level::Level::KillAllKruskal2D(::level::kill_all_kruskal::Conf2D {
                        size: (conf.maze_size as isize * 2+1, conf.maze_size as isize * 2+1),
                        percent: conf.percent as f64,
                        bug: (
                            if conf.x_shift { 1 } else { 0 },
                            if conf.y_shift { 1 } else { 0 },
                        ),
                        entities,
                    }).create(world);
                },
            }

            world.maintain();
        }
    }
}
