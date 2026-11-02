use bevy::{
    app::Plugin,
    ecs::{hierarchy::Children, system::Commands},
    log::info,
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};

use crate::InGameScreen;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::WorldMap), enter_world_map);
    }
}

/// When moving to the world map, run this system once to setup everything.
///
/// This is the entrypoint as well, so have to consider it's not doint something idk
fn enter_world_map(mut commands: Commands) {
    info!("Entering world map");
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.)
        }
        Children [
            Text("Hello, WorldMap")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::WorldMap)
    });
}
