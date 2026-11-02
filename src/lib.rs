use bevy::{
    DefaultPlugins,
    app::{App, Plugin, Startup},
    camera::Camera2d,
    ecs::system::{Commands, ResMut},
    prelude::States,
    scene::{CommandsSceneExt, bsn},
    state::{
        app::AppExtStates,
        state::{NextState, SubStates},
    },
};

mod crafting;
mod encounters;
mod nights;
mod world_map;

pub struct JamPlugin;

impl Plugin for JamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);

        app.add_plugins((
            crafting::CraftingPlugin,
            encounters::EncounterPlugin,
            nights::NightsPlugin,
            world_map::WorldMapPlugin,
        ));

        app.init_state::<AppState>().add_sub_state::<InGameScreen>();

        app.add_systems(Startup, spawn_camera)
            .add_systems(Startup, debug_start_game);
    }
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

fn debug_start_game(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::InGame);
}
