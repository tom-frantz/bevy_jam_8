use bevy::{
    app::{Plugin, Startup, Update},
    asset::{Asset, AssetApp, AssetServer, Assets, Handle},
    ecs::{
        hierarchy::Children,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    log::info,
    reflect::TypePath,
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};
use serde::Deserialize;

use crate::{InGameScreen, encounters::deserialize::TomlEncLoader};

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

use crate::crafting::Item;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct Encounter {
    pub name: String,
    pub context_items: Vec<Context>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Context {
    pub name: String,
    pub text: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub text: String,
    pub goto_context: Option<String>, // TODO how to map this to a context.name?
    pub requires: Option<Vec<Item>>,
    pub execute: Option<Vec<Execute>>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Execute {
    AddItem(Item),
    LoseItem(Item),
    Leave,
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
