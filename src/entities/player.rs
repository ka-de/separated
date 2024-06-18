use bevy::{ ecs::{ bundle::Bundle, component::Component }, sprite::SpriteBundle };
use bevy_ecs_ldtk::{ EntityInstance, LdtkEntity, Worldly };

use crate::{
    components::{
        armor::Armor,
        health::Health,
        collision::ColliderBundle,
        ground::GroundDetection,
        items::Items,
        climbing::Climber,
        swimming::Swimmer,
    },
    plugins::input,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_bundle("player.png")]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    pub climber: Climber,
    pub swimmer: Swimmer,
    pub ground_detection: GroundDetection,
    pub health: Health,
    pub armor: Armor,

    // Build Items Component manually by using `impl From<&EntityInstance>`
    #[from_entity_instance]
    items: Items,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,

    // Input manager components
    #[with(make_action_map)]
    input_map: input::InputMap,
    action_state: input::ActionState,
    action_timers: input::ActionTimers,
}

fn make_action_map(_: &EntityInstance) -> input::InputMap {
    input::make_action_map()
}
