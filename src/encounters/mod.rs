use bevy::{
    app::{Plugin, Startup, Update},
    asset::{Asset, AssetApp, AssetServer, Assets, Handle},
    color::{Color, palettes::tailwind},
    ecs::{
        component::Component,
        entity::Entity,
        event::EntityEvent,
        hierarchy::Children,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, In, IntoSystem, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    log::{debug, info},
    picking::{
        Pickable,
        events::{PointerLeave, PointerOver, PointerPress},
    },
    reflect::TypePath,
    scene::{CommandsSceneExt, EntityCommandsSceneExt, Scene, bsn, on},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    text::TextColor,
    ui::{BackgroundColor, FlexDirection, Node, UiRect, Val, widget::Text},
};
use serde::Deserialize;

use crate::{
    InGameScreen,
    crafting::{AspectValue, Aspects, InventoryItem, ItemAspect},
    encounters::deserialize::TomlEncLoader,
};

mod deserialize;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Encounters>();
        app.init_resource::<CurrentEncounter>();
        app.init_asset::<Encounter>();
        app.init_asset_loader::<TomlEncLoader>();
        app.add_systems(OnEnter(InGameScreen::Encounter), enter_encounter_screen);
        app.add_systems(Startup, load_assets);
        app.add_systems(Update, check_encounter_input.pipe(on_choice_select));
    }
}
const KEYS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

fn check_encounter_input(
    input: Res<ButtonInput<KeyCode>>,
    current_encounter: Res<CurrentEncounter>,
) -> Option<Choice> {
    let choices: &Vec<Choice> = &current_encounter.current_context.choices;
    for (index, choice) in choices.iter().enumerate() {
        if input.just_pressed(KEYS[index]) {
            debug!("Got input");
            return Some(choice.clone());
        }
    }
    None
}

use crate::crafting::Item;

#[derive(Asset, TypePath, Default, Clone, Debug, Deserialize)]
pub struct Encounter {
    pub name: String,
    pub context_items: Vec<Context>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Context {
    pub name: String,
    pub text: String,
    pub choices: Vec<Choice>,
}

#[derive(Component, Default, Debug, Deserialize, Clone)]
pub struct Choice {
    pub text: String,
    pub goto_context: Option<String>,
    pub requires: Option<Requires>,
    pub execute: Option<Vec<Execute>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Requires {
    alt_text: String,
    req_items: Vec<Item>,
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

// TODO make this an option?
#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentEncounter {
    encounter: Encounter,
    current_context: Context,
}

#[derive(Component, Debug, Clone, Default)]
struct ChatBoxMarker;

fn on_choice_select(
    choice_selected: In<Option<Choice>>,
    mut current_encounter: ResMut<CurrentEncounter>,
    mut commands: Commands,
    mut q: Query<Entity, With<ChatBoxMarker>>,
    mut items: Query<&mut InventoryItem>,
) {
    if let Some(choice_selected) = &*choice_selected {
        handle_choice(
            choice_selected.clone(),
            current_encounter,
            commands,
            q,
            items,
        )
    }
}

/// When moving to the encounter screen, run this system once to setup everything.
fn enter_encounter_screen(
    mut commands: Commands,
    encounter: Res<Encounters>,
    mut current_encounter: ResMut<CurrentEncounter>,
    encounters: Res<Assets<Encounter>>,
    mut items: Query<(&mut InventoryItem)>,
) {
    let enc = encounters
        .get(&encounter.handle)
        .expect("ahhhhh too many frogs");
    current_encounter.encounter = enc.clone();
    let initial_context = enc.context_items.first().expect("AHH");
    current_encounter.current_context = initial_context.clone();
    let my_enc = set_scene_context(initial_context, items);

    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(40.))
        }
        Children [
            Node {
                flex_grow: 1.0
            }
            Children [
                Text("Background goes here.")
            ]
            --
            @my_enc
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Encounter)
    });
}

fn set_scene_context(context: &Context, mut items: Query<(&mut InventoryItem)>) -> impl Scene {
    let choices = context
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| (index + 1, choice))
        .map(|(index, choice)| {
            let fulfilled_requirements = if let Some(reqs) = &choice.requires {
                reqs.req_items.iter().all(|req_item| {
                    items
                        .iter_mut()
                        .find(|inventory_item| {
                            (&inventory_item.item == req_item) && (inventory_item.amount > 0)
                        })
                        .is_some()
                })
            } else {
                true
            };
            choice_scene(index, choice, fulfilled_requirements)
        })
        .collect::<Vec<_>>();

    bsn! {
        Node {
            width: Val::Px(600.),
            padding: UiRect::all(Val::Px(40.)),
            flex_direction: FlexDirection::Column
        }
        BackgroundColor(tailwind::GRAY_600)
        ChatBoxMarker
        Children [
            @flavour_text(context.text.clone())
            // --
            // @action_text("This is something that happened to your character")
            --
            {choices}
        ]
    }
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

fn choice_scene(index: usize, choice: &Choice, fulfilled_requirements: bool) -> impl Scene {
    let text = if !fulfilled_requirements {
        // Should exist
        let reqs = choice.requires.clone().unwrap();
        reqs.alt_text
    } else {
        choice.text.clone()
    };

    let enumarated_choice_text = format!("{index}. {text}");
    let c = choice.clone();
    bsn! {
        Pickable
        c
        Text(enumarated_choice_text)
        TextColor(Color::srgb(1.0, 1.0, 1.0))
        on(choice_click_observer)
        on(choice_pointer_enter_observer)
        on(choice_pointer_exit_observer)
    }
}

fn choice_click_observer(
    event: On<PointerPress>,
    choices: Query<&Choice>,
    mut current_encounter: ResMut<CurrentEncounter>,
    mut commands: Commands,
    mut q: Query<Entity, With<ChatBoxMarker>>,
    mut items: Query<&mut InventoryItem>,
) {
    let selected_choice_entity = event.event_target();
    let selected_choice = choices.get(selected_choice_entity).expect("exists").clone();
    handle_choice(selected_choice, current_encounter, commands, q, items)
}

fn choice_pointer_enter_observer(
    event: On<PointerOver>,
    mut choices: Query<(&Choice, &mut TextColor)>,
) {
    let mut selected_choice_entity = event.event_target();
    let (_, mut text) = choices.get_mut(selected_choice_entity).expect("exists");
    *text = TextColor(Color::srgb(0.8, 0.5, 0.2));
}

fn choice_pointer_exit_observer(
    event: On<PointerLeave>,
    mut choices: Query<(&Choice, &mut TextColor)>,
) {
    let mut selected_choice_entity = event.event_target();
    let (_, mut text) = choices.get_mut(selected_choice_entity).expect("exists");
    *text = TextColor(Color::srgb(1.0, 1.0, 1.0));
}

fn handle_choice(
    choice_selected: Choice,
    mut current_encounter: ResMut<CurrentEncounter>,
    mut commands: Commands,
    mut q: Query<Entity, With<ChatBoxMarker>>,
    mut items: Query<(&mut InventoryItem)>,
) {
    if let Some(requires) = &choice_selected.requires {
        info!("Requirements: {requires:?}");
        // check if the player has the requirements
        let has_requirements = requires.req_items.iter().all(|req_item| {
            items
                .iter_mut()
                .find(|inventory_item| {
                    (&inventory_item.item == req_item) && (inventory_item.amount > 0)
                })
                .is_some()
        });
        if !has_requirements {
            return;
        }
    }

    if let Some(execute) = &choice_selected.execute {
        for action in execute {
            match action {
                Execute::AddItem(item) => {
                    info!("Got item: {item:?}");
                    if let Some(mut existing_item) = items
                        .iter_mut()
                        .find(|inventory_item| &inventory_item.item == item)
                    {
                        existing_item.amount += 1;
                    } else {
                        let new_inv_item = InventoryItem {
                            amount: 1,
                            item: item.clone(),
                            // TODO figure out how to get aspects
                            aspects: ItemAspect {
                                primary: AspectValue(Aspects::Earth, 1),
                                secondary: None,
                                tertiary: None,
                            },
                        };
                        commands.spawn_empty().insert(new_inv_item);
                    }
                }
                Execute::LoseItem(item) => {
                    info!("Lost item: {item:?}");
                    if let Some(mut existing_item) = items
                        .iter_mut()
                        .find(|inventory_item| &inventory_item.item == item)
                    {
                        if existing_item.amount > 0 {
                            existing_item.amount -= 1;
                        }
                    }
                }
                Execute::Leave => todo!(),
            }
        }
    }

    if let Some(goto_context) = &choice_selected.goto_context {
        info!("Goto: {goto_context:?}");
        let target_context = current_encounter
            .encounter
            .context_items
            .iter()
            .find(|item| &item.name == goto_context)
            .expect("should exist")
            .clone();
        current_encounter.current_context = target_context.clone();
        let chatbox = q.single_mut().expect("Should be only one chatbox");
        let new_context_scene = set_scene_context(&target_context, items);

        commands
            .entity(chatbox)
            .despawn_children()
            .apply_scene(new_context_scene);
    }
}
