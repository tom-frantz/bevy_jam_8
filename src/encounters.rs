use bevy::{
    app::Plugin,
    ecs::{hierarchy::Children, system::Commands},
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};

use crate::InGameScreen;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::Encounter), enter_encounter_screen);
    }
}

/// When moving to the encounter screen, run this system once to setup everything.
fn enter_encounter_screen(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.)
        }
        Children [
            Text("Hello, Encounter Screen")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Encounter)
    });
}
