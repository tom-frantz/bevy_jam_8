use bevy::{
    DefaultPlugins,
    app::{App, Plugin},
    prelude::States,
    state::{app::AppExtStates, state::SubStates},
};

pub struct JamPlugin;

impl Plugin for JamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);

        app.init_state::<AppState>().add_sub_state::<InGameScreen>();
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
///
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::InGame)]
enum InGameScreen {
    #[default]
    WorldMap,
    Encounter,
    NightTime,
    Crafting,
}
