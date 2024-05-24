use bevy::prelude::*;

use crate::components::{GravityScale, Velocity};

const GRAVITY: f32 = 19.61;
const TILE_SIZE: f32 = 32.0;

// 💀 Zone
#[derive(Component)]
struct DeathZone {
    size: Vec2,
}

// Component representing a tile in the game
#[derive(Component)]
struct Tile {
    size: Vec2,
    ground: bool,
}

// Default implementation for Tile
impl Default for Tile {
    fn default() -> Self {
        Self {
            size: Vec2::new(TILE_SIZE, TILE_SIZE),
            ground: true,
        }
    }
}

// Function to create a bundle with the DeathZone component and a SpriteBundle
pub fn death_zone_bundle() -> impl Bundle {
    (
        DeathZone {
            size: Vec2::new(800.0, 50.0),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),          // red color for debug
                custom_size: Some(Vec2::new(800.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
            ..default()
        },
    )
}

// Function to create a bundle with the Tile component and a SpriteBundle
pub fn tile_bundle(tile_position: Vec2) -> impl Bundle {
    (
        Tile::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(tile_position.extend(0.0)),
            ..default()
        },
    )
}

// System to handle collisions between entities and the death zone
fn handle_death_zone_collisions(
    mut commands: Commands,
    death_zone_query: Query<(&DeathZone, &Transform)>,
    entity_query: Query<(Entity, &Transform, &Sprite, &Velocity, &Name)>,
) {
    for (death_zone, death_zone_transform) in death_zone_query.iter() {
        let death_zone_position = death_zone_transform.translation.truncate();
        let death_zone_size = death_zone.size;

        for (entity, transform, sprite, _, name) in entity_query.iter() {
            let entity_position = transform.translation.truncate();
            let entity_size = sprite.custom_size.unwrap_or(Vec2::splat(1.0));

            if is_colliding(
                entity_position,
                entity_size,
                death_zone_position,
                death_zone_size,
            ) {
                info!("Collision detected between entity {} and death zone", name);
                commands.entity(entity).despawn();
            }
        }
    }
}

// System to apply gravity to all entities with a Velocity and GravityScale component
fn apply_gravity(mut query: Query<(&mut Velocity, &GravityScale)>, time: Res<Time>) {
    for (mut velocity, gravity_scale) in query.iter_mut() {
        velocity.y -= GRAVITY * gravity_scale.0 * time.delta_seconds();
    }
}

// System to handle collisions between "animal" entities and tiles
fn handle_collisions(
    mut animal_query: Query<(&mut Velocity, &mut Transform, &Sprite, &GravityScale)>,
    tile_query: Query<(&Tile, &Transform), Without<Velocity>>,
    time: Res<Time>,
) {
    for (mut animal_velocity, mut animal_transform, animal_sprite, gravity_scale) in animal_query.iter_mut() {
        let animal_size = animal_sprite.custom_size.unwrap_or(Vec2::splat(1.0));
        let mut collision_resolved = false;

        for (tile, tile_transform) in tile_query.iter() {
            let tile_position = tile_transform.translation.truncate();
            let tile_size = tile.size;

            if tile.ground
                && is_colliding(
                    animal_transform.translation.truncate(),
                    animal_size,
                    tile_position,
                    tile_size,
                )
            {
                let tile_top = tile_position.y + tile_size.y / 2.0;
                let animal_bottom = animal_transform.translation.y - animal_size.y / 2.0;
                let penetration_depth = animal_bottom - tile_top;

                if penetration_depth < 0.0 {
                    animal_velocity.y = 0.0;
                    animal_transform.translation.y += -penetration_depth * gravity_scale.0;
                    collision_resolved = true;
                    break;
                }
            }
        }

        if !collision_resolved {
            animal_velocity.y -= GRAVITY * gravity_scale.0 * time.delta_seconds();
        }
    }
}

// Helper function to check if two rectangles are colliding
fn is_colliding(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let a_min = a_pos - a_size / 2.0;
    let a_max = a_pos + a_size / 2.0;
    let b_min = b_pos - b_size / 2.0;
    let b_max = b_pos + b_size / 2.0;

    a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
}

// Function to add the gravity, collision, and death zone collision systems to the Bevy app
pub fn setup_world_systems(app: &mut App) -> &mut App {
    app.add_systems(Update, apply_gravity)
        .add_systems(Update, handle_collisions)
        .add_systems(Update, handle_death_zone_collisions)
}
