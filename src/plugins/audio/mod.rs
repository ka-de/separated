use bevy::{ audio::{ DefaultSpatialScale, SpatialScale }, prelude::* };

mod delayed_audio_source;
mod insert_audio_components;
mod play_background_music;
pub(crate) mod change_global_volume;

const AUDIO_SCALE: f32 = 1.0 / 25.0;

pub fn plugin(app: &mut App) {
    app.add_plugins(delayed_audio_source::plugin)
        .add_systems(PreUpdate, (
            insert_audio_components::insert_spatial_listener,
            insert_audio_components::insert_audio_sources,
        ))
        .insert_resource(GlobalVolume::new(1.0))
        .insert_resource(DefaultSpatialScale(SpatialScale::new_2d(AUDIO_SCALE)));
}
