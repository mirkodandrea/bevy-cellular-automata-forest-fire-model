use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileVisible};

use bevy::input::mouse::MouseMotion;
use bevy::{input::Input, math::Vec3, render::camera::Camera};

use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Reflect)]
enum VegetationState {
    Green,
    Burning,
    Empty,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Reflect)]
pub struct FFMCellState {
    state: VegetationState,
}

#[derive(Component)]
pub struct LastUpdate(f64);

// A simple camera system for moving and zooming the camera.
#[allow(dead_code)]
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

// A system that moves the camera based on mouse motion
pub fn camera_movement(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &OrthographicProjection), With<Camera>>,
) {
    // Get the primary window
    // Iterate over mouse motion events
    for event in mouse_motion_events.iter() {
        // Get the delta movement of the mouse
        let delta = event.delta;

        // Iterate over camera entities
        for (mut transform, ortho) in query.iter_mut() {
            // Convert the delta movement to world coordinates
            let delta_world = Vec3::new(delta.x, -delta.y, 0.0) * ortho.scale;

            // Update the camera position
            transform.translation += delta_world;
        }
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 256, y: 256 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let state = match rand::thread_rng().gen_bool(0.5) {
                true => VegetationState::Green,
                false => VegetationState::Empty,
            };
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        visible: TileVisible(true),
                        ..Default::default()
                    },
                    FFMCellState { state },
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
            i += 1;
        }
    }

    let tile_size = TilemapTileSize { x: 4.0, y: 4.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(
        (TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        }),
    );
}

fn update_graphics(mut query: Query<(Entity, &FFMCellState, &mut TileColor)>) {
    for (entity, state, mut color) in query.iter_mut() {
        // change tile color based on state
        match state.state {
            VegetationState::Green => {
                color.0 = Color::GREEN;
            }
            VegetationState::Burning => {
                color.0 = Color::RED;
            }
            VegetationState::Empty => {
                color.0 = Color::BLACK;
            }
        }
    }
}

fn update_burning(
    mut query: Query<(Entity, &TilePos, &mut FFMCellState)>,
    mut tile_storage_query: Query<(&TileStorage, &TilemapSize)>,
) {
    let (tile_storage, map_size) = tile_storage_query.single_mut();

    // First Pass: Collect entities to update without mutating anything
    let mut to_burn = Vec::new();
    let mut to_green = Vec::new();
    let mut to_empty = Vec::new();
    for (entity, position, state) in query.iter() {
        match state.state {
            VegetationState::Green => {
                let neighbor_count =
                    Neighbors::get_square_neighboring_positions(position, map_size, true)
                        .entities(tile_storage)
                        .iter()
                        .filter(|neighbor| {
                            let neighbour_cell =
                                query.get_component::<FFMCellState>(**neighbor).unwrap();
                            neighbour_cell.state == VegetationState::Burning
                        })
                        .count();

                if neighbor_count > 0 {
                    to_burn.push(entity);
                } else {
                    if rand::thread_rng().gen_bool(0.001) {
                        to_burn.push(entity);
                    }
                }
            }
            VegetationState::Burning => {
                to_empty.push(entity);
            }
            VegetationState::Empty => {
                if rand::thread_rng().gen_bool(0.01) {
                    to_green.push(entity);
                }
            }
        }
    }

    // Second Pass: Mutate states based on collected data
    for entity in to_burn {
        if let Ok((_entity, _position, mut state)) = query.get_mut(entity) {
            state.state = VegetationState::Burning;
        }
    }

    for entity in to_green {
        if let Ok((_entity, _position, mut state)) = query.get_mut(entity) {
            state.state = VegetationState::Green; // Assuming the transition is to Empty
        }
    }

    for entity in to_empty {
        if let Ok((_entity, _position, mut state)) = query.get_mut(entity) {
            state.state = VegetationState::Empty;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Game of Life Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, movement)
        // .add_systems(Update, camera_movement)
        .add_systems(Update, update_graphics)
        .add_systems(Update, update_burning)
        .run();
}
