use util::ConvCoord;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Conf {
    pub size: (isize, isize),
    pub percent: f64,
    pub bug: (isize, isize),
    pub entities: HashMap<::entity::EntityConf, usize>,
}

impl Conf {
    pub fn create(&self, world: &mut ::specs::World) {
        let (turrets, entities) = self.entities.iter()
            .fold((0, 0), |mut acc, (e, nbr)| {
                if e.is_turret_like() {
                    acc.0 += nbr;
                } else {
                    acc.1 += nbr;
                }
                acc
            });

        let maze = ::level::KruskalDecorated::new(
            ::na::Vector2::new(self.size.0, self.size.1),
            self.percent,
            ::na::Vector2::new(self.bug.0, self.bug.1),
            turrets,
            entities,
        );

        // Build walls
        let mut maze_colors = HashMap::new();
        maze_colors.insert(maze.start_cell, ::CONFIG.start_color);
        maze_colors.insert(maze.end_cell, ::CONFIG.end_color);

        ::entity::create_2d_maze_walls_w(&maze_colors, &maze.maze, world);

        // Build teleport
        ::entity::create_teleport_w(
            ::na::Isometry3::new(
                maze.maze.to_world(&maze.end_cell),
                (maze.end_opening - maze.end_cell).axis_angle_z(),
            ),
            maze.maze.scale,
            world,
        );

        // Build player
        let dir = maze.start_opening - maze.start_cell;
        let player_pos = maze.maze.to_world(&maze.start_cell)
            - 0.2 * ::na::Vector3::new(dir[0] as f32, dir[1] as f32, 0.0);
        world.write_resource::<::resource::PlayerControl>().pointer =
            [(-dir[1] as f32).atan2(dir[0] as f32), 0.0];
        ::entity::create_player_w(player_pos, false, world);

        // Build turrets
        self.entities.iter()
            .filter(|&(e, _)| e.is_turret_like())
            .flat_map(|(e, &nbr)| {
                let mut v = vec![];
                v.resize(nbr, e);
                v
            })
            .zip(maze.turret_cells.iter())
            .for_each(|(conf, cell)| {
                let pos = maze.maze.to_world(cell);
                conf.create(pos, world);
            });

        // Build entities
        self.entities.iter()
            .filter(|&(e, _)| !e.is_turret_like())
            .flat_map(|(e, &nbr)| {
                let mut v = vec![];
                v.resize(nbr, e);
                v
            })
            .zip(maze.entity_cells.iter())
            .for_each(|(conf, cell)| {
                let pos = maze.maze.to_world(cell);
                conf.create(pos, world);
            });

        // Build maze resource
        world.add_resource(::resource::Maze::Maze2D(maze.maze));
    }
}