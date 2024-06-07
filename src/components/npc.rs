use bevy::{
    ecs::{ bundle::Bundle, component::Component, query::Added },
    log::info,
    prelude::Query,
    sprite::SpriteBundle,
};
use bevy_ecs_ldtk::{ prelude::{ LdtkEntity, LdtkFields }, EntityInstance };

use super::{ collision::ColliderBundle, patrol::Patrol };

// Npc
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Npc;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct NpcBundle {
    #[sprite_bundle("npc.png")]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub npc: Npc,
    #[from_entity_instance]
    pub entity: EntityInstance,
}

// NpcPatrol
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct NpcPatrol;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct NpcPatrolBundle {
    #[sprite_bundle("npc.png")]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub npc: Npc,
    pub npcpatrol: NpcPatrol,
    #[ldtk_entity]
    pub patrol: Patrol,
    #[from_entity_instance]
    pub entity: EntityInstance,
}

pub fn print_npc_info(query: Query<&EntityInstance, Added<Npc>>) {
    for ldtk_entity in query.iter() {
        if let Ok(name) = ldtk_entity.get_string_field("name") {
            info!("Npc name={:?}", name);

            if let Ok(has_dialogue) = ldtk_entity.get_bool_field("hasDialogue") {
                info!("Npc {} hasDialogue: {}", name, has_dialogue);
            }
        }
    }
}

pub fn print_npcpatrol_info(query: Query<&EntityInstance, Added<NpcPatrol>>) {
    for ldtk_entity in query.iter() {
        if let Ok(name) = ldtk_entity.get_string_field("name") {
            info!("NpcPatrol name: {}", name);

            if let Ok(has_dialogue) = ldtk_entity.get_bool_field("hasDialogue") {
                info!("NpcPatrol {} hasDialogue: {}", name, has_dialogue);
            }
        }
    }
}
