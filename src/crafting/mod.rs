use bevy::{
    app::{Plugin, Startup},
    asset::{AssetApp, AssetServer, Handle, LoadedFolder},
    color::palettes::tailwind,
    ecs::{
        component::Component,
        hierarchy::Children,
        resource::Resource,
        system::{Commands, ResMut},
    },
    scene::{CommandsSceneExt, bsn},
    state::{
        app::AppExtStates,
        state::{ComputedStates, OnEnter, States},
        state_scoped::DespawnOnExit,
    },
    ui::{BackgroundColor, FlexDirection, Node, Val, widget::Text},
};
use serde::Deserialize;

use crate::{
    InGameScreen,
    crafting::assets::{Ingredient, IngredientAssetLoader, Ingredients, Recipe},
};

mod assets;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<Ingredients>()
            .init_asset::<Recipe>()
            .init_asset_loader::<IngredientAssetLoader>()
            .init_asset_loader::<IngredientAssetLoader>()
            .add_systems(Startup, load_crafting_assets);

        app.init_state::<CraftingState>()
            .add_computed_state::<ChosenCatalystState>();
        app.add_systems(OnEnter(InGameScreen::Crafting), enter_crafting_screen);
    }
}

#[derive(Resource, Debug)]
pub struct IngredientsAsset(Handle<Ingredients>);
#[derive(Resource, Debug)]
pub struct RecipeAssets(Handle<LoadedFolder>);

//
// INGREDIENT ZONE
//

/// The type of an item. This is not very useful in of itself, but is a 'factor'
/// as to what an item actually is.
#[derive(Copy, Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub enum Item {
    Water,
    Oil,

    FireCrystal,
    PixieDust,
    FrogTears,

    ArcaneSolute,
    GlowMoss,

    DivinationOil,
    CrystalLens,

    MagicLens,
    Heliopal,

    // TODO remove
    Mushroom,
    Stone,
}

/// A list of aspects an item can possess. This is just the name of the aspect
/// itself, the value must be associated with it
#[derive(Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub enum Aspects {
    Fire,
    Earth,
    Air,
    Water,
}

/// An aspect and a level associated with it. More useful.
#[derive(Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub struct AspectValue(pub Aspects, pub u8);

/// Describes all the aspects, and their levels for a given item.
#[derive(Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub struct ItemAspect {
    pub primary: AspectValue,
    pub secondary: Option<AspectValue>,
    pub tertiary: Option<AspectValue>,
}

// What item, with what aspects, and how much of it is available.
#[derive(Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub struct InventoryItem {
    pub amount: usize,
    pub item: Item,
}

//
// CRAFTING WITH CATALYSTS
//

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub enum Catalysts {
    #[default]
    Refine,
    Combine,
    Subtract,
}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct RefineState {
    items: Vec<InventoryItem>,
}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct CombineState {}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct SubtractState {}

/// Manages the players chosen catalyst and also the ingredients and any jazz
/// necessary for crafting recipes.
#[derive(Clone, PartialEq, Eq, Hash, Debug, States, Deserialize)]
pub enum CraftingState {
    Refine(RefineState),
    Combine(CombineState),
    Subtract(SubtractState),
}

/// Use this when determining state changes between different catalysts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct ChosenCatalystState(Catalysts);

impl ComputedStates for ChosenCatalystState {
    /// We set the source state to be the state, or a tuple of states,
    /// we want to depend on. You can also wrap each state in an Option,
    /// if you want the computed state to execute even if the state doesn't
    /// currently exist in the world.
    type SourceStates = CraftingState;

    /// We then define the compute function, which takes in
    /// your SourceStates
    fn compute(sources: CraftingState) -> Option<Self> {
        match sources {
            CraftingState::Refine(_) => Some(Self(Catalysts::Refine)),
            CraftingState::Combine(_) => Some(Self(Catalysts::Combine)),
            CraftingState::Subtract(_) => Some(Self(Catalysts::Subtract)),
        }
    }
}

impl Default for CraftingState {
    fn default() -> Self {
        Self::Refine(RefineState::default())
    }
}

fn load_crafting_assets(asset_server: ResMut<AssetServer>, mut commands: Commands) {
    commands.insert_resource(IngredientsAsset(asset_server.load("ingredients.ing.toml")));
    commands.insert_resource(RecipeAssets(asset_server.load_folder("recipes")));
}

/// When moving to the crafting screen, run this system once to setup everything.
fn enter_crafting_screen(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Row
        }
        Children [
            Node {
                width: Val::Percent(50.),
            }
            BackgroundColor(tailwind::BLUE_500)
            Children [
                Text("Philosopher's StoneZone")
            ]
            --
            Node {
                width: Val::Percent(50.),
            }
            BackgroundColor(tailwind::GRAY_600)
            Children [
                Text("BOok of ingredients")
            ]
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Crafting)
    });
}
