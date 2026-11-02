use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetApp, AssetServer, Assets, Handle},
    ecs::{
        hierarchy::Children,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    log::info,
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};

use crate::{
    InGameScreen,
    encounters::deserialize::{Encounter, TomlEncLoader},
};

mod deserialize;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Encounters>();
        app.init_asset::<Encounter>();
        app.init_asset_loader::<TomlEncLoader>();
        app.add_systems(OnEnter(InGameScreen::Encounter), enter_encounter_screen);
        app.add_systems(Startup, load_assets);
        app.add_systems(Update, print_on_load);
    }
}

#[derive(Resource, Default)]
struct Encounters {
    handle: Handle<Encounter>,
}

fn load_assets(mut encounters: ResMut<Encounters>, asset_server: Res<AssetServer>) {
    encounters.handle = asset_server.load("encounters/encounter_template.enc.toml")
}

fn print_on_load(mut encounters: ResMut<Encounters>, custom_assets: Res<Assets<Encounter>>) {
    let enc = custom_assets.get(&encounters.handle);

    if enc.is_none() {
        info!("Custom Asset Not Ready");
        return;
    }
    let e = enc.unwrap();
    info!(e.name);
    for context in &e.context_items {
        info!(context.name);
        info!(context.text);
        for choice in &context.choices {
            info!(choice.text)
        }
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
