use bevy::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_life::{
    CellState, CellularAutomatonPlugin, MooreCell2d, SimulationBatch, SimulationPause,
};
use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Reflect)]
enum VegetationState {
    Green,
    Burning,
    Empty,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Reflect)]
pub struct VegatationCellState {
    state: VegetationState,
    coords: IVec2,
}

const RANDOM_FIRE_OCCURRENCE: f64 = 0.001;
const RANDOM_REGROWTH_OCCURRENCE: f64 = 0.01;

impl CellState for VegatationCellState {
    fn new_cell_state<'a>(&self, neighbor_cells: impl Iterator<Item = &'a Self>) -> Self {
        let burning_cell_count = neighbor_cells
            .filter(|&c| c.state == VegetationState::Burning)
            .count();

        match self.state {
            VegetationState::Green => {
                if burning_cell_count > 0 {
                    VegatationCellState {
                        state: VegetationState::Burning,
                        coords: self.coords,
                    }
                } else {
                    let random_fire_occurs = rand::thread_rng().gen_bool(RANDOM_FIRE_OCCURRENCE);
                    VegatationCellState {
                        state: if random_fire_occurs {
                            VegetationState::Burning
                        } else {
                            VegetationState::Green
                        },
                        coords: self.coords,
                    }
                }
            }
            VegetationState::Burning => VegatationCellState {
                state: VegetationState::Empty,
                coords: self.coords,
            },
            VegetationState::Empty => {
                let random_regrowth_occurs =
                    rand::thread_rng().gen_bool(RANDOM_REGROWTH_OCCURRENCE);

                VegatationCellState {
                    state: if random_regrowth_occurs {
                        VegetationState::Green
                    } else {
                        VegetationState::Empty
                    },
                    coords: self.coords,
                }
            }
        }
    }

    fn color(&self) -> Option<Color> {
        match self.state {
            VegetationState::Green => Some(Color::rgb(0., 1., 0.)),
            VegetationState::Burning => Some(Color::rgb(1., 0., 0.)),
            VegetationState::Empty => None,
        }
    }
}

pub type VegetationAutomataPlugin = CellularAutomatonPlugin<MooreCell2d, VegatationCellState>;

fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());
}

fn setup_map(mut commands: Commands) {
    let commands = &mut commands;

    let mut rng = rand::thread_rng();
    let (size_x, size_y) = (600, 800);
    let sprite_size = 2.0;
    let color = Color::rgba(0., 0., 0., 0.);

    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(
            -(size_x as f32 * sprite_size) / 2.,
            -(size_y as f32 * sprite_size) / 2.,
            0.,
        )))
        .with_children(|builder| {
            for y in 0..=size_y {
                for x in 0..=size_x {
                    let state = match rng.gen_bool(0.3) {
                        true => VegetationState::Green,
                        false => VegetationState::Empty,
                    };
                    // let state = match (rng.gen_bool(0.01), state) {
                    //     (true, VegetationState::Green) => VegetationState::Burning,
                    //     _ => state,
                    // };

                    let cell = VegatationCellState {
                        state: state,
                        coords: IVec2::new(x, y),
                    };
                    builder.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(sprite_size)),
                                color,
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                sprite_size * x as f32,
                                sprite_size * y as f32,
                                0.,
                            ),
                            ..default()
                        },
                        MooreCell2d::new(IVec2::new(x, y)),
                        cell,
                    ));
                }
            }
        });
    println!("map generated");
}

fn toggle_simulation_pause_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    simulation_pause_query: Option<Res<SimulationPause>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if simulation_pause_query.is_some() {
            // If the SimulationPause resource exists, remove it to resume the simulation
            commands.remove_resource::<SimulationPause>();
        } else {
            // If the SimulationPause resource does not exist, add it to pause the simulation
            commands.insert_resource(SimulationPause);
        }
    }
}

fn main() {
    App::new()
        .register_type::<VegatationCellState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game Of Life".to_string(),
                resolution: [1200.0, 800.0].into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VegetationAutomataPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(SimulationBatch)
        .add_systems(Startup, (setup_camera, setup_map))
        .add_systems(Update, (toggle_simulation_pause_system,))
        .run();
}
