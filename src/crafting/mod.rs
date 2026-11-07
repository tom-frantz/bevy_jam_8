use bevy::{
    app::{Plugin, Startup},
    asset::{AssetApp, AssetServer, Handle, LoadedFolder},
    color::palettes::css,
    ecs::{
        component::Component,
        entity::Entity,
        event::EntityEvent,
        hierarchy::Children,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
        template::FromTemplate,
    },
    log::info,
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{
            PointerClick, PointerDragDrop, PointerDragEnd, PointerDragEnter, PointerDragStart,
        },
    },
    scene::{CommandsSceneExt, Scene, bsn, bsn_list, on},
    sprite::Sprite,
    state::{
        app::AppExtStates,
        state::{ComputedStates, NextState, OnEnter, State, States},
        state_scoped::DespawnOnExit,
    },
    text::TextColor,
    transform::components::Transform,
    ui::{
        FlexDirection, Node,
        Val::{self},
        widget::{ImageNode, Text},
    },
    ui_widgets::Button,
};
use serde::Deserialize;

use crate::{
    InGameScreen,
    crafting::assets::{IngredientAssetLoader, Ingredients, Recipe},
    draggable::Draggable,
    drawing::DrawingPath,
    tracked_ui_node::TransformPropogatedUiNode,
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
    Struby,
    Heliopal,
    Mushroom,
}

impl Item {
    pub fn sprite(&self) -> impl Scene {
        use Item::*;

        let (dimensions, path): (Vec2, &str) = match self {
            Struby => (Vec2::new(208., 293.), "struby.png"),
            Heliopal => (Vec2::new(0.0, 0.0), "heliopal.png"),
            Mushroom => (Vec2::new(0.0, 0.0), "mushroom.png"),
        };

        let image = format!("ingredient_images/{path}");

        bsn! {
            Sprite {
                image: image,
                custom_size: dimensions
            }
        }
    }
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
    None,

    Refine,
    Combine,
    Subtract,
}

impl Catalysts {
    pub fn sprite(&self) -> impl Scene {
        let asset_path = match self {
            Catalysts::None => "philosophers_stone.png",
            Catalysts::Refine => "philosophers_stone_earth.png",
            Catalysts::Combine => "philosophers_stone_water.png",
            Catalysts::Subtract => "philosophers_stone_fire.png",
        };

        bsn! {
            Sprite {
                image: asset_path,
                custom_size: Vec2::new(150., 449.)
            }
        }
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct RefineState {
    items: Vec<Item>,
}

impl RefineState {
    fn add_ingredient(&self, ingredient: Item) -> Self {
        let mut new = self.clone();
        new.items.push(ingredient);
        new
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct CombineState {}

impl CombineState {
    fn add_ingredient(&self, ingredient: Item) -> Self {
        let mut new = self.clone();
        todo!();
        new
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct SubtractState {}

impl SubtractState {
    fn add_ingredient(&self, ingredient: Item) -> Self {
        let mut new = self.clone();
        todo!();
        new
    }
}

/// Manages the players chosen catalyst and also the ingredients and any jazz
/// necessary for crafting recipes.
#[derive(Clone, PartialEq, Eq, Hash, Debug, States, Deserialize)]
pub enum CraftingState {
    Refine(RefineState),
    Combine(CombineState),
    Subtract(SubtractState),
}

impl CraftingState {
    pub fn reset_crafting_state(&self) -> Self {
        use CraftingState::*;

        match self {
            Refine(_) => Refine(Default::default()),
            Combine(_) => Combine(Default::default()),
            Subtract(_) => Subtract(Default::default()),
        }
    }

    pub fn add_ingredient(&self, ingredient: Item) -> Self {
        use CraftingState::*;

        match self {
            Refine(state) => Refine(state.add_ingredient(ingredient)),
            Combine(state) => Combine(state.add_ingredient(ingredient)),
            Subtract(state) => Subtract(state.add_ingredient(ingredient)),
        }
    }

    pub fn try_craft(&self) -> Result<Self, ()> {
        dbg!(self);
        Err(())
    }
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
    commands.spawn_scene_list(bsn_list! {
        @ingredient_scene(Item::Struby)
        --
        @ingredient_scene(Item::Struby)
        --
        @ingredient_scene(Item::Struby)
        --
        @book_scene()
        // --
        // @page_scene()
        --
        @philosophers_stone_scene(Catalysts::Refine)
    });
}

#[derive(Component, FromTemplate, Clone, Debug)]
struct BookContainerEntity(Entity);

fn book_scene() -> impl Scene {
    const LEFT_PAGE_X_OFFSET: f32 = -300.0;
    const RIGHT_PAGE_X_OFFSET: f32 = 0.0;
    const TOP_PAGE_Y_OFFSET: f32 = 200.0;
    const MID_PAGE_Y_OFFSET: f32 = 0.0;

    // TODO sync with new asset when we get it

    bsn! {
        @Draggable
        Sprite { image: "book-68.png"}
        Transform {
            translation: Vec3::new(0., 0., -10.0)
        }
        #BookContainer
        Children [
            #LeftPageTop
            Transform {
                translation: Vec3::new(LEFT_PAGE_X_OFFSET, TOP_PAGE_Y_OFFSET, 0.0)
            }
            @page_scene()
            --
            #LeftPageBottom
            Transform {
                translation: Vec3::new(LEFT_PAGE_X_OFFSET, MID_PAGE_Y_OFFSET, 0.0)
            }
            @page_scene()
            --
            #RightPageTop
            Transform {
                translation: Vec3::new(RIGHT_PAGE_X_OFFSET, TOP_PAGE_Y_OFFSET, 0.0)
            }
            @page_scene()
            --
            #RightPageBottom
            Transform {
                translation: Vec3::new(RIGHT_PAGE_X_OFFSET, MID_PAGE_Y_OFFSET, 0.0)
            }
            @page_scene()
            --
            Transform {
                translation: Vec3::new(LEFT_PAGE_X_OFFSET, -175.0, 0.0)
            }
            TransformPropogatedUiNode [
                Button
                Node
                Children [
                    Text("< Back")
                    @book_text()
                ]
                BookContainerEntity(#BookContainer)
                on(|click: On<PointerClick>, book_container: Query<&BookContainerEntity>, mut commands: Commands | {
                    let entity = book_container.get(click.event_target()).expect("Must have `BookContainerEntity` on button");
                    dbg!(entity);
                })
            ]
            --
            Transform {
                translation: Vec3::new(300., -150.0, 0.0)
            }
            TransformPropogatedUiNode [
                Button
                Node
                Children [
                    Text("Next >")
                    @book_text()
                ]
            ]

        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Crafting)
    }
}

fn book_text() -> impl Scene {
    bsn! {
        TextColor(css::BLACK)
    }
}

#[derive(Component, Default, Clone, Debug)]
struct IngredientNoteNode;

fn page_scene() -> impl Scene {
    bsn! {
        TransformPropogatedUiNode [
            IngredientNoteNode
            Node {
                top: Val::Px(50.),
                left: Val::Px(50.)
            }
            Children [
                Node {
                    max_width: Val::Px(100.),
                    max_height: Val::Px(100.),
                    flex_direction: FlexDirection::Row
                }
                ImageNode {
                    image: "ingredient_images/struby.png"
                }
                --
                Node {
                    flex_direction: FlexDirection::Column
                }
                Children [
                    Text("Title")
                    @book_text()
                    --
                    Text("Description")
                    @book_text()
                    --
                    Text("aspects")
                    @book_text()
                ]
            ]
        ]
    }
}

#[derive(Component, Default, Clone)]
pub struct AddedToCrafting;

fn ingredient_scene(item: Item) -> impl Scene {
    let item_sprite = item.sprite();
    bsn! {
        Transform {
            translation: Vec3::new(0.0, 0.0, 10.0)
        }
        @Draggable
        @item_sprite
        item
        on(on_ingredient_drag_through)
        DespawnOnExit<InGameScreen>(InGameScreen::Crafting)
    }
}

fn philosophers_stone_scene(catalyst: Catalysts) -> impl Scene {
    let sprite = catalyst.sprite();
    bsn! {
        // @Draggable
        @sprite
        Pickable
        on(on_philosophers_start_drag)
        on(on_philosophers_end_drag_successful)
        on(on_philosophers_end_drag_finalise)
        Children [
            #TopPoint
            // on(on_philosophers_start_drag)
            --
            #BottomPoint
            // on(on_philosophers_end_drag)
        ]
    }
}

fn on_philosophers_start_drag(
    _event: On<PointerDragStart>,
    crafting_state: Res<State<CraftingState>>,
    mut next_state: ResMut<NextState<CraftingState>>,
) {
    info!("on_philosophers_start_drag");
    next_state.set(crafting_state.reset_crafting_state());
}

fn on_ingredient_drag_through(
    event: On<PointerDragEnter>,
    mut commands: Commands,
    ingredient: Query<(&Item, Option<&AddedToCrafting>)>,
    crafting_state: Res<State<CraftingState>>,
    mut next_state: ResMut<NextState<CraftingState>>,
) {
    info!("on_ingredient_drag_through");
    let ingredient_entity = event.event_target();
    let (ingredient, already_added) = ingredient
        .get(ingredient_entity)
        .expect("No ingredient found from drag through");

    if already_added.is_some() {
        return;
    }

    // Add the ingredient, and make sure it can't be added again
    commands.entity(ingredient_entity).insert(AddedToCrafting);
    next_state.set(crafting_state.add_ingredient(*ingredient))
}

fn on_philosophers_end_drag_successful(
    _event: On<PointerDragDrop>,
    crafting_state: Res<State<CraftingState>>,
) {
    info!("on_philosophers_end_drag_successful");
    let crafting = crafting_state.try_craft();
    dbg!(crafting);
}

/// Called after `on_philosophers_end_drag_successful`, regardless of it it reaches back to the right spot
fn on_philosophers_end_drag_finalise(
    event: On<PointerDragEnd>,
    mut commands: Commands,
    ingredients: Query<Entity, With<AddedToCrafting>>,
) {
    info!("on_philosophers_end_drag_finalise");

    for entity in &ingredients {
        commands.entity(entity).remove::<AddedToCrafting>();
    }
}
