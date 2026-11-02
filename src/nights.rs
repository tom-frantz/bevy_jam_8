use bevy::{
    app::Plugin,
    ecs::{hierarchy::Children, system::Commands},
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};

use crate::InGameScreen;

pub struct NightsPlugin;

impl Plugin for NightsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::NightTime), enter_night_screen);
    }
}

/// When moving to the nighttime screen, run this system once to setup everything.
fn enter_night_screen(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.)
        }
        Children [
            Text("Hello, Night time screen")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::NightTime)
    });
}
