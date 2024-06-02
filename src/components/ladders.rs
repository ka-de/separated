use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkIntCell;
use std::collections::HashSet;

use super::sensorbundle::SensorBundle;

// Attach this to any component to allow the player (or any climber entity) to climb up and
// down on it.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Climbable;

// LdtkIntCell representing a ladder in the game world.
#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct LadderBundle {
    #[from_int_grid_cell]
    pub sensor_bundle: SensorBundle,
    pub climbable: Climbable,
}

// Attach this component to any entity to allow them to climb up ladders.
#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Climber {
    pub climbing: bool,
    pub intersecting_climbables: HashSet<Entity>,
}
