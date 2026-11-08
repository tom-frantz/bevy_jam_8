use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

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
        template::{FromTemplate, template},
    },
    log::info,
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{
            PointerClick, PointerDragDrop, PointerDragEnd, PointerDragEnter, PointerDragStart,
        },
    },
    scene::{CommandsSceneExt, EntityCommandsSceneExt, Scene, bsn, bsn_list, on},
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

        #[cfg(feature = "dev")]
        {
            app.add_systems(Startup, load_debug_ingredients);
        }
        app.init_state::<CraftingState>()
            .add_computed_state::<ChosenCatalystState>();
        app.add_systems(OnEnter(InGameScreen::Crafting), enter_crafting_screen);
    }
}

// Debug startup things
fn load_debug_ingredients(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list! {
        template(|_| Ok( InventoryItem {
            item: Item::Struberry,
            amount: 0
        } ))
        --
        template(|_| Ok( InventoryItem {
            item: Item::CrystalWater,
            amount: 0
        } ))
        --
        template(|_| Ok( InventoryItem {
            item: Item::FrogSlime,
            amount: 0
        } ))
        --
        template(|_| Ok( InventoryItem {
            item: Item::WorldTreeBlood,
            amount: 0
        } ))
        --
        template(|_| Ok( InventoryItem {
            item: Item::TaintedWorldTreeBlood,
            amount: 0
        } ))
        --
        template(|_| Ok( InventoryItem {
            item: Item::FastJuice ,
            amount: 0
        } ))
    });
}

/// `#[allow(unused)]` is here because these just keep the handles loaded
#[derive(Resource, Debug)]
pub struct IngredientsAsset(#[allow(unused)] Handle<Ingredients>);
#[derive(Resource, Debug)]
pub struct RecipeAssets(#[allow(unused)] Handle<LoadedFolder>);

//
// INGREDIENT ZONE
//

/// The type of an item. This is not very useful in of itself, but is a 'factor'
/// as to what an item actually is.
#[derive(Copy, Clone, Debug, Component, PartialEq, Eq, Hash, Deserialize)]
pub enum Item {
    Struberry,
    CrystalWater,
    FrogSlime,
    WorldTreeBlood,
    TaintedWorldTreeBlood,
    AngryMushrooms,
    FastJuice,
    SleepJuice,
    LastBreathOfInnocent,
    BillowOfAFreeSpirit,
    WillOWisps,
    Golem,
    TreeMedicine,
    IgnisAster,
    CloudCandy,
    LuminoTruant,
    Mushroom, // Remove this in favour of angry mushroom?
    StopSand,
    SleepGas,
    MagicLens,
    BoomBottle,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        f.write_str(match self {
            Struberry => "Struberry",
            CrystalWater => "Crystal Water",
            FrogSlime => "Frog Slime",
            WorldTreeBlood => "World Tree Blood",
            TaintedWorldTreeBlood => "Tainted World Tree Blood",
            AngryMushrooms => "Angry Mushrooms",
            FastJuice => "Fast Juice",
            SleepJuice => "Sleep Juice",
            LastBreathOfInnocent => "Last Breath Of Innocent",
            BillowOfAFreeSpirit => "Billow Of A Free Spirit",
            WillOWisps => "Will o' Wisps",
            Golem => "Golem",
            TreeMedicine => "Tree Medicine",
            IgnisAster => "Ignis Aster",
            CloudCandy => "Cloud Candy",
            LuminoTruant => "Lumino Truant",
            Mushroom => todo!(),
            StopSand => "Stop Sand",
            SleepGas => "Sleep Gas",
            MagicLens => "Magic Lens",
            BoomBottle => "Boom Bottle",
        })
    }
}

impl Item {
    pub fn sprite(&self) -> impl Scene {
        use Item::*;

        let (dimensions, path): (Vec2, &str) = match self {
            Struberry => (Vec2::new(208., 293.), "struby.png"),
            CrystalWater => (Vec2::new(285., 258.), "crystal_water.png"),
            FrogSlime => (Vec2::new(184., 275.), "frog_slime.png"),
            WorldTreeBlood => (Vec2::new(282.0, 334.0), "world_tree_lifeblood_good.png"),
            TaintedWorldTreeBlood => (Vec2::new(268., 326.), "world_tree_lifeblood_bad.png"),
            AngryMushrooms => (Vec2::new(000., 000.), "TODO_TODO_TODO.png"),
            FastJuice => (Vec2::new(193., 269.), "fast_juice.png"),
            SleepJuice => (Vec2::new(299., 331.), "sleep_juice.png"),
            LastBreathOfInnocent => (Vec2::new(169., 246.), "crab_breath_bad.png"),
            BillowOfAFreeSpirit => (Vec2::new(371., 363.), "crab_breath_good.png"),
            WillOWisps => (Vec2::new(000., 000.), "TODO_TODO_TODO.png"),
            Golem => (Vec2::new(208., 325.), "golem.png"),
            TreeMedicine => (Vec2::new(193., 269.), "tree_medicine.png"),
            IgnisAster => (Vec2::new(211., 352.), "ignis_aster.png"),
            CloudCandy => (Vec2::new(000., 000.), "TODO_TODO_TODO.png"),
            LuminoTruant => (Vec2::new(262., 244.), "lumino_truant.png"),
            Mushroom => (Vec2::new(000., 000.), "TODO_TODO_TODO.png"),
            StopSand => (Vec2::new(230., 464.), "stop_sand.png"),
            SleepGas => (Vec2::new(294., 567.), "sleep_gas.png"),
            MagicLens => (Vec2::new(476., 681.), "magic_lens.png"),
            BoomBottle => (Vec2::new(563., 421.), "boom_bottle.png"),
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

/// What item, with what aspects, and how much of it is available.
///
/// # SAFETY
/// Do not despawn these entities, ever! If the player runs out, simply set `item`
/// to `0`. I'm going to use these to track what the player has discovered.
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
        @ingredient_scene(Item::Struberry)
        --
        @ingredient_scene(Item::CrystalWater)
        --
        @ingredient_scene(Item::FastJuice)
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

/// Starts at zero, goes forwards from that.
#[derive(Component, FromTemplate, Default, Clone, Debug)]
struct BookPageCount(usize);

impl Deref for BookPageCount {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BookPageCount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// TODO sync with new asset when we get it
const LEFT_PAGE_X_OFFSET: f32 = -300.0;
const RIGHT_PAGE_X_OFFSET: f32 = 0.0;
const TOP_PAGE_Y_OFFSET: f32 = 200.0;
const MID_PAGE_Y_OFFSET: f32 = 0.0;

const TOP_LEFT_PAGE: Vec3 = Vec3::new(LEFT_PAGE_X_OFFSET, TOP_PAGE_Y_OFFSET, 0.0);
const TOP_RIGHT_PAGE: Vec3 = Vec3::new(LEFT_PAGE_X_OFFSET, MID_PAGE_Y_OFFSET, 0.0);
const BOTTOM_LEFT_PAGE: Vec3 = Vec3::new(RIGHT_PAGE_X_OFFSET, TOP_PAGE_Y_OFFSET, 0.0);
const BOTTOM_RIGHT_PAGE: Vec3 = Vec3::new(RIGHT_PAGE_X_OFFSET, MID_PAGE_Y_OFFSET, 0.0);

fn book_scene() -> impl Scene {
    bsn! {
        @Draggable
        Sprite { image: "book-68.png"}
        Transform {
            translation: Vec3::new(0., 0., -10.0)
        }
        #BookContainer
        BookPageCount
        Children [
            #LeftPageTop
            Transform {
                translation: TOP_LEFT_PAGE
            }
            @page_scene()
            --
            #LeftPageBottom
            Transform {
                translation: TOP_RIGHT_PAGE
            }
            @page_scene()
            --
            #RightPageTop
            Transform {
                translation: BOTTOM_LEFT_PAGE
            }
            @page_scene()
            --
            #RightPageBottom
            Transform {
                translation: BOTTOM_RIGHT_PAGE
            }
            @page_scene()
            --
            Transform {
                translation: Vec3::new(LEFT_PAGE_X_OFFSET, -175.0, 0.0)
            }
            TransformPropogatedUiNode [
                Button
                BookContainerEntity(#BookContainer)
                @turn_page_observer(PageDirection::Forward)
                Node
                Children [
                    Text("Previous Page")
                    @book_text()
                ]
            ]
            --
            Transform {
                translation: Vec3::new(300., -150.0, 0.0)
            }
            TransformPropogatedUiNode [
                Button
                BookContainerEntity(#BookContainer)
                @turn_page_observer(PageDirection::Backwanrs)
                Node
                Children [
                    Text("Next Page")
                    @book_text()
                ]
            ]

        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Crafting)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PageDirection {
    Forward,
    Backwanrs,
}

fn turn_page_observer(page_direction: PageDirection) -> impl Scene {
    let observer = move |click: On<PointerClick>,
                         book_container_entity: Query<&BookContainerEntity>,
                         mut book_container: Query<(&Children, &mut BookPageCount)>,
                         ingredient_ui_node: Query<(Entity), With<IngredientNoteNode>>,
                         mut commands: Commands,
                         items: Query<&InventoryItem>| {
        let entity = book_container_entity
            .get(click.event_target())
            .expect("Must have `BookContainerEntity` on button");
        let book_container_entity = entity.0;

        let (book_container_children, mut book_page_count) = book_container
            .get_mut(book_container_entity)
            .expect("book container must exist");

        const ITEMS_PER_PAGE: usize = 4;
        let total_discovered_items = items.count();

        book_page_count.0 = match page_direction {
            PageDirection::Forward => book_page_count.saturating_sub(1),
            PageDirection::Backwanrs => {
                (**book_page_count + 1).min(total_discovered_items / ITEMS_PER_PAGE)
            }
        };

        let items = items
            .iter()
            .skip(**book_page_count * ITEMS_PER_PAGE)
            .take(4)
            .cloned()
            .collect::<Vec<_>>();

        let mut existing_children: Vec<Entity> = Vec::default();
        // despawn old items
        for entity in ingredient_ui_node.iter_many(book_container_children.iter()) {
            if let Ok(entity) = entity {
                commands.entity(entity).despawn();
                info!("despawning")
            }

            if let Err(bevy::ecs::query::QueryEntityError::QueryDoesNotMatch(entity, _)) = entity {
                info!("Not existingly!");
                existing_children.push(entity);
            }
        }

        commands
            .entity(book_container_entity)
            .with_children(move |p| {
                for (pos, item) in vec![
                    TOP_LEFT_PAGE,
                    TOP_RIGHT_PAGE,
                    BOTTOM_LEFT_PAGE,
                    BOTTOM_RIGHT_PAGE,
                ]
                .into_iter()
                .zip(items.iter())
                {
                    p.spawn_empty().apply_scene(bsn! {
                        Transform {
                            translation: pos
                        }
                        @page_scene()
                    });
                }
            });
    };

    bsn! {
        on(observer)
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
        IngredientNoteNode
        TransformPropogatedUiNode [
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
    mut commands: Commands,
    crafting_state: Res<State<CraftingState>>,
    mut next_state: ResMut<NextState<CraftingState>>,
) {
    info!("on_philosophers_start_drag");
    next_state.set(crafting_state.reset_crafting_state());
    commands.insert_resource(DrawingPath::new());
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
    todo!()
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
    commands.remove_resource::<DrawingPath>();
}
