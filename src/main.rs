// Turn clippy into a real bitch
#![warn(clippy::all, clippy::pedantic)]

// This changes the executable to a graphical application instead of a CLI one
// only for Release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Provides functions to read and manipulate environment variables.
use std::env;

use bevy::ecs::system::EntityCommands;
use bevy::{ ecs::system::EntityCommand, render::settings::WgpuSettings };
use bevy::render::RenderPlugin;
use wgpu::Backends;

/// ⚠️ UI STUFF
use sickle_ui::ui_style::{ SetNodeBottomExt as _, SetNodeLeftExt as _ };
use sickle_ui::{
    ui_builder::{ UiBuilder, UiBuilderExt, UiRoot },
    ui_commands::SetTextExt as _,
    ui_style::{
        SetBackgroundColorExt,
        SetImageExt as _,
        SetNodeAlignSelfExt as _,
        SetNodeHeightExt,
        SetNodeJustifyContentsExt as _,
        SetNodePositionTypeExt as _,
        SetNodeRightExt as _,
        SetNodeTopExt as _,
        SetNodeWidthExt,
    },
    widgets::prelude::*,
};
/// ⚠️ END OF UI STUFF

mod components;
mod plugins;

use bevy::prelude::*;
use bevy_tweening::*;
// Steamworks
use bevy_steamworks::*;

/////////////////////////////////////////////////////////////////////////////
/// ⚠️ UI TEST ⚠️
/////////////////////////////////////////////////////////////////////////////

/// SetFont
struct SetFont(String, f32, Color);

impl EntityCommand for SetFont {
    fn apply(self, entity: Entity, world: &mut World) {
        let asset_server = world.resource::<AssetServer>();
        let font = asset_server.load(&self.0);

        if let Some(mut text) = world.entity_mut(entity).get_mut::<Text>() {
            for text_section in &mut text.sections {
                text_section.style.font = font.clone();
                text_section.style.font_size = self.1;
                text_section.style.color = self.2;
            }
        }
    }
}

/// BannerWidget
#[derive(Component)]
pub struct BannerWidget;

/// BannerLabel
/// A marker component used internally to initialize the label font.
#[derive(Component)]
struct BannerLabel;

/// BannerWidgetConfig
pub struct BannerWidgetConfig {
    pub label: String,
    pub font: String,
    pub font_size: f32,
}

impl BannerWidgetConfig {
    pub fn from(
        label: impl Into<String>,
        font: impl Into<String>,
        font_size: impl Into<f32>
    ) -> Self {
        Self {
            label: label.into(),
            font: font.into(),
            font_size: font_size.into(),
        }
    }
}

pub trait UiBannerWidgetExt<'w, 's> {
    fn banner_widget<'a>(&'a mut self, config: BannerWidgetConfig) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiBannerWidgetExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot> {
    fn banner_widget<'a>(
        &'a mut self,
        config: BannerWidgetConfig
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container((ImageBundle::default(), BannerWidget), |banner| {
            banner
                .style()
                .position_type(PositionType::Absolute)
                // Center the children (the label) horizontally.
                .justify_content(JustifyContent::Center)
                .width(Val::Px(100.0))
                .height(Val::Px(12.0))
                // Add a nice looking background image to our widget.
                .image("ui/label_gradient.png");

            // And we'll want a customizable label on the banner.
            let mut label = banner.label(LabelConfig::default());

            label
                .style()
                // Align the label relative to the top of the banner.
                .align_self(AlignSelf::Start)
                // Move us a few pixels down so we look nice relative to our font.
                .top(Val::Px(3.0));

            // We would like to set a default text style without having to pass in the AssetServer.
            label
                .entity_commands()
                .insert(BannerLabel)
                .set_text(config.label, None)
                .font(config.font, config.font_size, Color::rgb(1.0, 1.0, 1.0));
        })
    }
}

/// BannerWidgetCommands
///
/// An extension trait that exposes the SetFont command.
pub trait BannerWidgetCommands<'a> {
    fn font(
        &'a mut self,
        font: impl Into<String>,
        size: f32,
        color: Color
    ) -> &mut EntityCommands<'a>;
}

impl<'a> BannerWidgetCommands<'a> for EntityCommands<'a> {
    fn font(
        &'a mut self,
        font: impl Into<String>,
        size: f32,
        color: Color
    ) -> &mut EntityCommands<'a> {
        self.add(SetFont(font.into(), size, color))
    }
}

/// release_label
///
/// Prints out release or debug build on the UI.
fn release_label(mut commands: Commands) {
    // Print out "DEVELOPMENT BUILD" when not in release mode.
    #[cfg(debug_assertions)]
    commands
        .ui_builder(UiRoot)
        .banner_widget(BannerWidgetConfig::from("DEVELOPMENT BUILD", "fonts/bahnschrift.ttf", 8.0))
        .style()
        .left(Val::Px(100.0))
        .bottom(Val::Px(100.0));
    // ⚠️ TODO: This will have to go away from the actual release build
    // Print out "ALPHA RELEASE BUILD" when in release mode.
    #[cfg(not(debug_assertions))]
    commands
        .ui_builder(UiRoot)
        .banner_widget(
            BannerWidgetConfig::from("ALPHA RELEASE BUILD", "fonts/bahnschrift.ttf", 8.0)
        )
        .style()
        .left(Val::Px(100.0))
        .bottom(Val::Px(100.0));
}
/// END OF UI TEST ⚠️
/////////////////////////////////////////////////////////////////////////////

// Used for setting the Window icon
use bevy::winit::WinitWindows;
use winit::window::Icon;

// ⚠️ TODO: This will need to get eventually removed from main.
// RANDOM GAMEPLAY COMPONENTS
// use components::player::Player;
use components::torch::Torch;

fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icon.png").expect("Failed to open icon path").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

// ⚠️ TODO: Move audio stuff to its own thing
use bevy::audio::{ SpatialScale, AudioPlugin };
use bevy::audio::Volume;

const AUDIO_SCALE: f32 = 1.0 / 100.0;

fn change_global_volume(mut volume: ResMut<GlobalVolume>) {
    volume.volume = Volume::new(0.5);
}

// ⚠️ TODO: Currently very dumb, just plays one music on repeat!
fn play_background_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    // Create an entity dedicated to playing our background music
    commands.spawn(AudioBundle {
        source: asset_server.load("music/garam_masala_wip.ogg"),
        settings: PlaybackSettings::LOOP,
    });
}

// ⚠️ TODO: This is at the moment just testing Spatial Audio
//
//
fn play_2d_spatial_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn our emitter
    commands.spawn((
        Torch,
        AudioBundle {
            source: asset_server.load("vo/dogspeak.ogg"),
            settings: PlaybackSettings::LOOP, // ⚠️ TODO: Change it later to `ONCE` when done testing.
            //settings: PlaybackSettings::ONCE,
        },
    ));

    // Spawn our listener
    commands.spawn((
        SpatialListener::new(100.0), // Gap between the ears
        SpatialBundle::default(),
    ));
}
// End of TODO

// Allow the user to set the WGPU_BACKEND but have sane defaults for each platform.
fn get_backend() -> Option<Backends> {
    // Check if the WGPU_BACKEND environment variable is set
    if let Ok(backend_str) = env::var("WGPU_BACKEND") {
        // Convert the environment variable value to a Backend
        match backend_str.to_lowercase().as_str() {
            "vulkan" => {
                return Some(Backends::VULKAN);
            }
            "dx12" | "direct3d12" => {
                return Some(Backends::DX12);
            }
            "metal" => {
                return Some(Backends::METAL);
            }
            _ => eprintln!("Unsupported backend: {}", backend_str),
        }
    }

    // If the environment variable is not set, use the default logic
    if cfg!(target_os = "linux") {
        Some(Backends::VULKAN)
    } else if cfg!(target_os = "windows") {
        Some(Backends::DX12)
    } else if cfg!(target_os = "macos") {
        Some(Backends::METAL)
    } else {
        panic!("Unsupported Operating System!");
    }
}

fn main() {
    #[cfg(not(debug_assertions))] // ⚠️ TODO: At some point we will need to dev with Steam.
    match SteamworksPlugin::init_app(981370) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    }

    let backend = get_backend();
    let mut app = App::new();

    //app.add_systems(Startup, play_background_audio);

    #[cfg(target_arch = "wasm32")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#separated-canvas".into()),
            ..default()
        }),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let window_plugin = WindowPlugin::default();

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(AssetMetaCheck::Never);

    app.insert_resource(Msaa::Off) // Disable Multi-Sample Anti-Aliasing
        // DefaultPlugins
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: (WgpuSettings {
                    backends: backend,
                    ..default()
                }).into(),
                ..default()
            })
                .set(window_plugin)
                .set(ImagePlugin::default_nearest())
                .set(plugins::debug::make_log_plugin())
                // ⚠️ TODO: Maybe move this to its own thing? I'm not sure!
                .set(AudioPlugin {
                    default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                    ..default()
                }),
            // Tweening
            TweeningPlugin,
            plugins::gamestate::game_state_plugin,
            components::ui::setup_ui,
            components::systems::setup_ldtk,
            plugins::dialogueview::prelude::YarnSpinnerDialogueViewPlugin,
            plugins::debug::plugin,
        ))
        .add_systems(Startup, set_window_icon) // Set the Window icon.
        // UI TESTING ⚠️
        .add_systems(Startup, release_label)
        // AUDIO TESTING ⚠️
        .insert_resource(GlobalVolume::new(0.2)) // Set the GlobalVolume ⚠️ WIP
        .add_systems(Startup, change_global_volume); // Change the GlobalVolume ⚠️ WIP
    //.add_systems(Startup, play_2d_spatial_audio);

    app.run();
}
