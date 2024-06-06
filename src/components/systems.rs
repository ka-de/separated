use super::{
    items::Items,
    ladders::{ Climbable, Climber },
    misc::*,
    patrol::patrol,
    player::Player,
};
use crate::plugins::{ dialogueview::not_in_dialogue, gamestate::GameState, input::movement };

use std::collections::{ HashMap, HashSet };

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn setup_ldtk(app: &mut App) {
    app.register_ldtk_int_cell::<super::misc::WallBundle>(1)
        .register_ldtk_int_cell::<super::ladders::LadderBundle>(2)
        .register_ldtk_int_cell::<super::misc::WallBundle>(3)
        .register_ldtk_entity::<super::torch::TorchBundle>("Torch")
        .register_ldtk_entity::<super::player::PlayerBundle>("Player")
        .register_ldtk_entity::<super::enemy::MobBundle>("Mob")
        .register_ldtk_entity::<super::misc::ChestBundle>("Chest")
        .register_ldtk_entity::<super::misc::PumpkinsBundle>("Pumpkins")
        .add_systems(
            Update,
            (
                spawn_wall_collision,
                movement.run_if(not_in_dialogue),
                detect_climb_range,
                ignore_gravity_if_climbing,
                patrol,
                super::camera::fit_inside_current_level,
                update_level_selection,
                dbg_player_items,
                super::ground::spawn_ground_sensor,
                super::ground::ground_detection,
                super::ground::update_on_ground,
                restart_level,
            ).run_if(in_state(GameState::Playing))
        )
        .add_plugins((LdtkPlugin, RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)));
}

pub fn spawn_ldtk_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: use bevy_asset_loader states
    let ldtk_handle = asset_server.load("first_level.ldtk");
    commands.insert_resource(RapierConfiguration {
        gravity: Vec2::new(0.0, -2000.0),
        physics_pipeline_active: true,
        query_pipeline_active: true,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 1,
        },
        scaled_shape_subdivision: 10,
        force_update_from_transform_changes: false,
    });
    commands.insert_resource(LevelSelection::Uid(0));
    commands.insert_resource(LdtkSettings {
        level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
            load_level_neighbors: true,
        },
        set_clear_color: SetClearColor::FromLevelBackground,
        ..Default::default()
    });
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

pub fn dbg_player_items(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Items, &EntityInstance), With<Player>>
) {
    for (items, entity_instance) in &mut query {
        if input.just_pressed(KeyCode::KeyP) {
            dbg!(&items);
            dbg!(&entity_instance);
        }
    }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.iter().for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations.entry(grandparent.get()).or_default().insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&(GridCoords { x, y }))) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => {
                                plate_start = Some(x);
                            }
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| {
                                e.top += 1;
                            })
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(
                                Collider::cuboid(
                                    (((wall_rect.right as f32) - (wall_rect.left as f32) + 1.0) *
                                        (grid_size as f32)) /
                                        2.0,
                                    (((wall_rect.top as f32) - (wall_rect.bottom as f32) + 1.0) *
                                        (grid_size as f32)) /
                                        2.0
                                )
                            )
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(
                                Transform::from_xyz(
                                    (((wall_rect.left + wall_rect.right + 1) as f32) *
                                        (grid_size as f32)) /
                                        2.0,
                                    (((wall_rect.bottom + wall_rect.top + 1) as f32) *
                                        (grid_size as f32)) /
                                        2.0,
                                    0.0
                                )
                            )
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

pub fn detect_climb_range(
    mut climbers: Query<&mut Climber>,
    climbables: Query<Entity, With<Climbable>>,
    mut collisions: EventReader<CollisionEvent>
) {
    for collision in collisions.read() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b, _) => {
                if
                    let (Ok(mut climber), Ok(climbable)) = (
                        climbers.get_mut(*collider_a),
                        climbables.get(*collider_b),
                    )
                {
                    climber.intersecting_climbables.insert(climbable);
                }
                if
                    let (Ok(mut climber), Ok(climbable)) = (
                        climbers.get_mut(*collider_b),
                        climbables.get(*collider_a),
                    )
                {
                    climber.intersecting_climbables.insert(climbable);
                };
            }
            CollisionEvent::Stopped(collider_a, collider_b, _) => {
                if
                    let (Ok(mut climber), Ok(climbable)) = (
                        climbers.get_mut(*collider_a),
                        climbables.get(*collider_b),
                    )
                {
                    climber.intersecting_climbables.remove(&climbable);
                }

                if
                    let (Ok(mut climber), Ok(climbable)) = (
                        climbers.get_mut(*collider_b),
                        climbables.get(*collider_a),
                    )
                {
                    climber.intersecting_climbables.remove(&climbable);
                }
            }
        }
    }
}

pub fn ignore_gravity_if_climbing(
    mut query: Query<(&Climber, &mut GravityScale), Changed<Climber>>
) {
    for (climber, mut gravity_scale) in &mut query {
        if climber.climbing {
            gravity_scale.0 = 0.0;
        } else {
            gravity_scale.0 = 1.0;
        }
    }
}

pub fn update_level_selection(
    level_query: Query<(&LevelIid, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>
) {
    for (level_iid, level_transform) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        let level_bounds = Rect {
            min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
            max: Vec2::new(
                level_transform.translation.x + (level.px_wid as f32),
                level_transform.translation.y + (level.px_hei as f32)
            ),
        };

        for player_transform in &player_query {
            if
                player_transform.translation.x < level_bounds.max.x &&
                player_transform.translation.x > level_bounds.min.x &&
                player_transform.translation.y < level_bounds.max.y &&
                player_transform.translation.y > level_bounds.min.y &&
                !level_selection.is_match(&LevelIndices::default(), level)
            {
                *level_selection = LevelSelection::iid(level.iid.clone());
            }
        }
    }
}

pub fn restart_level(
    mut commands: Commands,
    level_query: Query<Entity, With<LevelIid>>,
    input: Res<ButtonInput<KeyCode>>
) {
    if input.just_pressed(KeyCode::KeyR) {
        for level_entity in &level_query {
            commands.entity(level_entity).insert(Respawn);
        }
    }
}
