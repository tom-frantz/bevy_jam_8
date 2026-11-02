use bevy::{
    a11y::ActionRequest,
    app::{Plugin, Startup, Update},
    asset::{Asset, AssetApp, AssetServer, Assets, Handle},
    color::palettes::tailwind,
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        query::With,
        resource::Resource,
        system::{Commands, In, IntoSystem, Query, Res, ResMut},
    },
    log::info,
    reflect::TypePath,
    scene::{CommandsSceneExt, Scene, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{BackgroundColor, Node, UiRect, Val, widget::Text},
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
        // app.add_systems(Update, get_some_choice_event.pipe(on_choice_select));
    }
}

use crate::crafting::Item;

#[derive(Asset, TypePath, Clone, Debug, Deserialize)]
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

impl Choice {
    pub fn when_chosen(&self) {}
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

#[derive(Resource, Debug, Clone)]
pub struct CurrentEncounter(Encounter);

fn get_some_choice_event(mut commands: Commands) -> Choice {
    Choice {
        text: "hello".to_string(),
        goto_context: None,
        requires: None,
        execute: None,
    }
}

#[derive(Component, Debug, Clone, Default)]
struct ChatBoxMarker;

fn on_choice_select(
    choice_selected: In<Choice>,
    current_encounter: ResMut<CurrentEncounter>,
    mut commands: Commands,
    q: Query<(Entity,), With<ChatBoxMarker>>,
) {
    let chatbox = q.single_mut();
    todo!("Fix me, Luke!")
}

/// When moving to the encounter screen, run this system once to setup everything.
fn enter_encounter_screen(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(40.))
        }
        Children [
            Node
            Children [
                Text("Background goes here.")
            ]
            --
            Node {
                width: Val::Px(400.)
            }
            BackgroundColor(tailwind::GRAY_600)
            ChatBoxMarker
            Children [
                @flavour_text("This is some yummy worldbuilding!")
                --
                @action_text("This is something that happened to your character")
                --
                @choice("Choice 1: Grumpy boys need a hug too")
                --
                @choice("Choice 2: fuckin' deck the mushroom")
            ]
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Encounter)
    });
}

fn flavour_text(msg: impl Into<String>) -> impl Scene {
    bsn! {
        Text(msg)
    }
}

fn action_text(msg: impl Into<String>) -> impl Scene {
    bsn! {
        Text(msg)

    }
}

fn choice(msg: impl Into<String>) -> impl Scene {
    bsn! {
        Text(msg)

    }
}
