use std::collections::HashMap;

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
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut, SystemParam},
    },
    input::{ButtonInput, keyboard::KeyCode},
    log::{info, warn},
    picking::{
        Pickable,
        events::{PointerLeave, PointerOver, PointerPress},
    },
    reflect::TypePath,
    scene::{CommandsSceneExt, EntityCommandsSceneExt, Scene, bsn, on},
    state::{
        condition::in_state,
        state::{NextState, OnEnter, State},
        state_scoped::DespawnOnExit,
    },
    text::TextColor,
    ui::{BackgroundColor, FlexDirection, Node, UiRect, Val, widget::Text},
};
use serde::Deserialize;

use crate::{
    InGameScreen, PlayerLocation,
    crafting::{InventoryItem, Item},
    encounters::{
        deserialize::TomlEncLoader,
        fancy_textify::{FancyTextColor, render_text_scene},
    },
    world_map::{IslandId, Islands},
};

mod deserialize;
mod fancy_textify;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Encounters>();
        app.init_resource::<EncounterState>();
        app.init_asset::<EncounterRes>();
        app.init_asset_loader::<TomlEncLoader>();
        app.add_systems(OnEnter(InGameScreen::Encounter), enter_encounter_screen);
        app.add_systems(Startup, load_assets);
        app.add_systems(
            Update,
            check_encounter_keyboard_input.run_if(in_state(InGameScreen::Encounter)),
        );
    }
}

#[derive(Asset, TypePath, Default, Clone, Debug, Deserialize)]
pub struct EncounterRes {
    // TODO print at top of screen?
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
    #[serde(default)]
    // Don't show this again if true
    pub is_completed: bool,
    pub goto_context: Option<String>,
    pub requires: Option<Requires>,
    pub requires_quest_state: Option<RequiresQuestState>,
    pub execute: Option<Vec<Execute>>,
}

impl Choice {
    fn is_visible(&self, quest_progress: &HashMap<IslandId, QuestProgress>) -> bool {
        if self.is_completed {
            return false;
        }
        match &self.requires_quest_state {
            None => true,
            Some(req) => {
                quest_progress
                    .get(&req.island.id())
                    .copied()
                    .unwrap_or_default()
                    == req.required_state
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Requires {
    alt_text: String,
    req_items: Vec<Item>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequiresQuestState {
    island: Islands,
    required_state: QuestProgress,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Execute {
    AddItem(Item),
    LoseItem(Item),
    MarkComplete,
    MarkQuest(QuestProgress),
    Leave,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestProgress {
    #[default]
    NotStarted,
    Started,
    GoodComplete,
    BadComplete,
}

impl QuestProgress {
    /// resource filename suffix
    fn file_suffix(self) -> &'static str {
        match self {
            Self::NotStarted => "1",
            Self::Started => "2",
            Self::GoodComplete => "3",
            Self::BadComplete => "3",
        }
    }
}

#[derive(Resource, Default)]
struct Encounters {
    handles: HashMap<(IslandId, QuestProgress), Handle<EncounterRes>>,
    quest_progress: HashMap<IslandId, QuestProgress>,
}

fn load_assets(mut encounters: ResMut<Encounters>, asset_server: Res<AssetServer>) {
    // load night
    let path = "encounters/night.enc.toml";
    encounters.handles.insert(
        (Islands::NIGHT.id(), QuestProgress::NotStarted),
        asset_server.load(path),
    );

    // load island 1
    for progress in [
        QuestProgress::NotStarted,
        QuestProgress::Started,
        QuestProgress::GoodComplete,
    ] {
        let path = format!("encounters/tree/tree_{}.enc.toml", progress.file_suffix());
        encounters
            .handles
            .insert((Islands::TREE.id(), progress), asset_server.load(path));
    }

    // load island 2
    for progress in [
        QuestProgress::NotStarted,
        QuestProgress::Started,
        QuestProgress::GoodComplete,
    ] {
        let path = format!("encounters/moon/moon_{}.enc.toml", progress.file_suffix());
        encounters
            .handles
            .insert((Islands::MOON.id(), progress), asset_server.load(path));
    }

    // load island 3
    for progress in [
        QuestProgress::NotStarted,
        QuestProgress::Started,
        QuestProgress::GoodComplete,
    ] {
        let path = format!("encounters/ruins/ruins_{}.enc.toml", progress.file_suffix());
        encounters
            .handles
            .insert((Islands::RUINS.id(), progress), asset_server.load(path));
    }

    // load island 4
    for progress in [
        QuestProgress::NotStarted,
        QuestProgress::Started,
        QuestProgress::GoodComplete,
    ] {
        let path = format!("encounters/party/party_{}.enc.toml", progress.file_suffix());
        encounters
            .handles
            .insert((Islands::PARTY.id(), progress), asset_server.load(path));
    }
}

impl Encounters {
    fn get_encounter_by_island_and_quest_progress(
        &self,
        island: IslandId,
        encounter_assets: &Assets<EncounterRes>,
    ) -> Option<EncounterRes> {
        let progress = self
            .quest_progress
            .get(&island)
            .copied()
            .unwrap_or_default();
        self.handles
            .get(&(island, progress))
            .and_then(|handle| encounter_assets.get(handle))
            .cloned()
    }
}

#[derive(SystemParam)]
struct ChoiceParams<'w, 's> {
    current_encounter: ResMut<'w, EncounterState>,
    encounters: ResMut<'w, Encounters>,
    encounter_assets: Res<'w, Assets<EncounterRes>>,
    next_location: ResMut<'w, NextState<PlayerLocation>>,
    next_screen: ResMut<'w, NextState<InGameScreen>>,
    commands: Commands<'w, 's>,
    chatbox: Query<'w, 's, Entity, With<ChatBoxMarker>>,
    items: Query<'w, 's, &'static mut InventoryItem>,
}

#[derive(Component, Debug, Clone, Copy)]
struct ChoiceIndex(usize);

#[derive(Component, Debug, Clone, Copy)]
struct ChoiceRequirementsMet(bool);

// TODO make this an option?
#[derive(Resource, Default, Debug, Clone)]
pub struct EncounterState {
    encounter: EncounterRes,
    // index into Encounter resource
    current_context_index: usize,
    island: IslandId,
}

impl EncounterState {
    fn current_context(&self) -> &Context {
        &self.encounter.context_items[self.current_context_index]
    }

    fn current_context_mut(&mut self) -> &mut Context {
        &mut self.encounter.context_items[self.current_context_index]
    }
}

#[derive(Component, Debug, Clone, Default)]
struct ChatBoxMarker;

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

fn check_encounter_keyboard_input(input: Res<ButtonInput<KeyCode>>, mut params: ChoiceParams) {
    // no current context? shouldn't happen
    if params.current_encounter.encounter.context_items.is_empty() {
        return;
    }

    let visible: Vec<usize> = params
        .current_encounter
        .current_context()
        .choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| choice.is_visible(&params.encounters.quest_progress))
        .map(|(real_index, _)| real_index)
        .collect();

    for (display_index, real_index) in visible.into_iter().enumerate().take(KEYS.len()) {
        if input.just_pressed(KEYS[display_index]) {
            return handle_choice(real_index, &mut params);
        }
    }
}

/// When moving to the encounter screen, run this system once to setup everything.
fn enter_encounter_screen(
    mut commands: Commands,
    encounters: Res<Encounters>,
    mut current_encounter: ResMut<EncounterState>,
    encounter_assets: Res<Assets<EncounterRes>>,
    location: Res<State<PlayerLocation>>,
    mut items: Query<&mut InventoryItem>,
) {
    let PlayerLocation::Island(island) = *location.get() else {
        info!("not on island");
        return;
    };

    let enc = encounters
        .get_encounter_by_island_and_quest_progress(island, &encounter_assets)
        .expect("no encounter for island + quest progress");

    current_encounter.encounter = enc;
    current_encounter.current_context_index = 0;
    current_encounter.island = island;
    let my_enc = set_scene_context(
        current_encounter.current_context(),
        &encounters.quest_progress,
        &mut items,
    );

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

fn set_scene_context(
    context: &Context,
    quest_progress: &HashMap<IslandId, QuestProgress>,
    items: &mut Query<&mut InventoryItem>,
) -> impl Scene {
    let choices = context
        .choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| choice.is_visible(quest_progress))
        .enumerate()
        .map(|(display_index, (real_index, choice))| {
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
            choice_scene(
                display_index + 1,
                real_index,
                choice,
                fulfilled_requirements,
            )
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
            // @action_text("This is something that happened to your character")
            --
            Text("\n")
            --
            {choices}
        ]
    }
}

fn flavour_text(msg: String) -> impl Scene {
    render_text_scene(msg)
}

fn choice_scene(
    display_index: usize,
    real_index: usize,
    choice: &Choice,
    fulfilled_requirements: bool,
) -> impl Scene {
    let (text, colour) = if !fulfilled_requirements {
        let reqs = choice
            .requires
            .clone()
            .expect("Didn't meet nonexistent requirements, uh oh!");
        (reqs.alt_text, Color::Srgba(tailwind::STONE_950))
    } else {
        (choice.text.clone(), Color::srgb(1.0, 1.0, 1.0))
    };

    let enumarated_choice_text = format!("{display_index}. {text}");
    let c = choice.clone();
    let choice_index = ChoiceIndex(real_index);
    let available = ChoiceRequirementsMet(fulfilled_requirements);
    bsn! {
        Pickable
        c
        choice_index
        available
        Text(enumarated_choice_text)
        TextColor(colour)
        on(choice_click_observer)
        on(choice_pointer_enter_observer)
        on(choice_pointer_exit_observer)
    }
}

/// Handler for clicking a choice
fn choice_click_observer(
    event: On<PointerPress>,
    choices: Query<&ChoiceIndex>,
    mut params: ChoiceParams,
) {
    let selected_choice_entity = event.event_target();
    let choice_index = choices
        .get(selected_choice_entity)
        .expect("No choice index fo choice")
        .0;
    handle_choice(choice_index, &mut params)
}

fn choice_pointer_enter_observer(
    event: On<PointerOver>,
    mut choices: Query<(&ChoiceRequirementsMet, &mut TextColor), With<Choice>>,
) {
    let selected_choice_entity = event.event_target();
    let (available, mut text) = choices
        .get_mut(selected_choice_entity)
        .expect("No choice present for hover target");
    if !available.0 {
        return;
    }
    *text = TextColor(FancyTextColor::HIGHLIGHT.to_bevy_color());
}

fn choice_pointer_exit_observer(
    event: On<PointerLeave>,
    mut choices: Query<(&ChoiceRequirementsMet, &mut TextColor), With<Choice>>,
) {
    let selected_choice_entity = event.event_target();
    let (available, mut text) = choices
        .get_mut(selected_choice_entity)
        .expect("No choice present for leave hover target");
    if !available.0 {
        return;
    }
    *text = TextColor(FancyTextColor::DEFAULT.to_bevy_color());
}

fn handle_choice(choice_index: usize, choice_params: &mut ChoiceParams) {
    let choice_selected =
        choice_params.current_encounter.current_context().choices[choice_index].clone();

    if !choice_selected.is_visible(&choice_params.encounters.quest_progress) {
        return;
    }

    if let Some(requires) = &choice_selected.requires {
        info!("Requirements: {requires:?}");
        // check if the player has the requirements
        let has_requirements = requires.req_items.iter().all(|req_item| {
            choice_params.items.iter().any(|inventory_item| {
                (&inventory_item.item == req_item) && (inventory_item.amount > 0)
            })
        });
        if !has_requirements {
            return;
        }
    }

    // do leave at the end
    let mut leave = false;
    if let Some(execute) = &choice_selected.execute {
        for action in execute {
            match action {
                Execute::AddItem(item) => {
                    info!("Got item: {item:?}");
                    if let Some(mut existing_item) = choice_params
                        .items
                        .iter_mut()
                        .find(|inventory_item| &inventory_item.item == item)
                    {
                        existing_item.amount += 1;
                    } else {
                        let new_inv_item = InventoryItem {
                            amount: 1,
                            item: item.clone(),
                        };
                        choice_params.commands.spawn_empty().insert(new_inv_item);
                    }
                }
                Execute::LoseItem(item) => {
                    info!("Lost item: {item:?}");
                    if let Some(mut existing_item) = choice_params
                        .items
                        .iter_mut()
                        .find(|inventory_item| &inventory_item.item == item)
                    {
                        if existing_item.amount > 0 {
                            existing_item.amount -= 1;
                        }
                    }
                }
                Execute::MarkComplete => {
                    choice_params
                        .current_encounter
                        .current_context_mut()
                        .choices[choice_index]
                        .is_completed = true;
                }
                Execute::MarkQuest(quest_state) => {
                    let island = choice_params.current_encounter.island;
                    choice_params
                        .encounters
                        .quest_progress
                        .insert(island, *quest_state);
                }
                Execute::Leave => leave = true,
            }
        }
    }

    if leave {
        if choice_params.current_encounter.island == Islands::NIGHT.id() {
            info!("sleepy time, back to map");
            choice_params.next_location.set(PlayerLocation::Home);
            choice_params.next_screen.set(InGameScreen::WorldMap);
            return;
        }

        info!("goto night time");
        let night_enc = choice_params
            .encounters
            .get_encounter_by_island_and_quest_progress(
                Islands::NIGHT.id(),
                &choice_params.encounter_assets,
            )
            .expect("night should be loaded");
        choice_params
            .next_location
            .set(PlayerLocation::Island(Islands::NIGHT.id()));
        choice_params.current_encounter.encounter = night_enc;
        choice_params.current_encounter.current_context_index = 0;
        choice_params.current_encounter.island = Islands::NIGHT.id();
    } else if let Some(goto_context) = &choice_selected.goto_context {
        info!("Goto: {goto_context:?}");
        let target_index = choice_params
            .current_encounter
            .encounter
            .context_items
            .iter()
            .position(|item| &item.name == goto_context)
            .expect("goto_target doesn't exist");
        choice_params.current_encounter.current_context_index = target_index;
    } else {
        return;
    }

    let chatbox = choice_params
        .chatbox
        .single()
        .expect("Should be only one chatbox");
    let new_context_scene = set_scene_context(
        choice_params.current_encounter.current_context(),
        &choice_params.encounters.quest_progress,
        &mut choice_params.items,
    );
    choice_params
        .commands
        .entity(chatbox)
        .despawn_children()
        .apply_scene(new_context_scene);
}
