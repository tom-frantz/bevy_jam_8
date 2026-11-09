use bevy::{
    DefaultPlugins,
    app::{App, Plugin, PluginGroup, Startup},
    asset::{AssetServer, Handle, LoadedFolder},
    camera::{Camera2d, ScalingMode::WindowSize},
    ecs::{
        component::Component,
        hierarchy::Children,
        observer::On,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    image::Image,
    picking::{Pickable, events::PointerClick},
    prelude::States,
    scene::{CommandsSceneExt, bsn, on},
    state::{
        app::AppExtStates,
        state::{NextState, OnEnter, SubStates},
    },
    ui::{AlignItems, FlexDirection, JustifyContent, Node, Val, ZIndex, widget::ImageNode},
    utils::default,
    window::{Window, WindowPlugin, WindowResolution},
    winit::WinitUserEvent::WindowAdded,
};

use crate::{
    asset_helpers::{DefaultPluginImageExt, set_default_font},
    world_map::Islands,
};

mod crafting;
mod encounters;
mod nights;
mod world_map;

mod asset_helpers;
mod camera_utils;
mod draggable;
mod drawing;
mod tracked_ui_node;

pub struct JamPlugin;

impl Plugin for JamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set_image_plugin().set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }));

        app.add_plugins((
            set_default_font,
            crafting::CraftingPlugin,
            encounters::EncounterPlugin,
            nights::NightsPlugin,
            world_map::WorldMapPlugin,
            draggable::DraggablePlugin,
            drawing::DrawingPlugin,
            tracked_ui_node::TrackedUiNodePlugin,
        ));

        app.init_state::<AppState>()
            .add_sub_state::<InGameScreen>()
            .add_sub_state::<PlayerLocation>();

        app.add_systems(Startup, (spawn_camera, load_assets))
            .add_systems(OnEnter(AppState::InGame), spawn_ui_overlay);

        app.add_systems(Startup, debug_start_game);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ImagesHandleHolder(#[allow(dead_code)] Vec<Handle<LoadedFolder>>);
fn load_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(ImagesHandleHolder(vec![
        asset_server.load_folder("final_assets"),
        asset_server.load_folder("backgrounds"),
        asset_server.load_folder("ingredient_images"),
    ]));
}

/// Controls the whole flow of the game.
///
/// `InGame` acts as a marker for when the player is playing, and then defers
/// to `InGameScreen` for the exact state of the UI displayed to the user.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    MainMenu,
    SettingsMenu,
    InGame,
}

/// Which screen is visible to the player at one point in time.
///
/// By having this a state, we ensure only one of these can be visible at a time.
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::InGame)]
enum InGameScreen {
    #[default]
    WorldMap,
    Encounter,
    NightTime,
    Crafting,
}

/// This actually makes shit show up (UI and 2D world)
fn spawn_camera(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Camera2d
    });
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct UiContainer;

fn spawn_ui_overlay(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        UiContainer
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart
        }
        ZIndex(1000)
        Pickable::IGNORE
        Children [
            ImageNode {
                image: "final_assets/philosophers_stone.png"
            }
            Node {
                max_width: Val::Px(100.)

            }
            on(|_: On<PointerClick>, mut screen: ResMut<NextState<InGameScreen>>| {
                screen.set(InGameScreen::Crafting);
            })
            --
            ImageNode {
                image: "map.png"
            }
            Node {
                max_width: Val::Px(100.)

            }
            on(|_: On<PointerClick>, mut screen: ResMut<NextState<InGameScreen>>| {
                screen.set(InGameScreen::WorldMap);
            })
            // --
            // ImageNode {
            //     image: "encounter.png"
            // }
            // Node {
            //     max_width: Val::Px(100.)

            // }
            // on(|_: On<PointerClick>, mut screen: ResMut<NextState<InGameScreen>>| {
            //     screen.set(InGameScreen::Encounter);
            // })
        ]
    });
}

fn debug_start_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut ingame_state: ResMut<NextState<InGameScreen>>,
) {
    app_state.set(AppState::InGame);
    ingame_state.set(InGameScreen::WorldMap);
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::InGame)]
enum PlayerLocation {
    Island(Islands),
    #[default]
    Home,
}
