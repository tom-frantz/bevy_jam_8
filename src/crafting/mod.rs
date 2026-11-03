use bevy::{
    app::Plugin,
    color::palettes::tailwind,
    ecs::{component::Component, hierarchy::Children, system::Commands},
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{BackgroundColor, FlexDirection, Node, Val, widget::Text},
};
use serde::Deserialize;

use crate::InGameScreen;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::Crafting), enter_crafting_screen);
    }
}

/// The type of an item. This is not very useful in of itself, but is a 'factor'
/// as to what an item actually is.
#[derive(Deserialize, Copy, Clone, Debug, Component, PartialEq, Eq)]
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
#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub enum Aspects {
    Fire,
    Earth,
    Air,
    Water,
}

/// An aspect and a level associated with it. More useful.
#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub struct AspectValue(pub Aspects, pub u8);

/// Describes all the aspects, and their levels for a given item.
#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub struct ItemAspect {
    pub primary: AspectValue,
    pub secondary: Option<AspectValue>,
    pub tertiary: Option<AspectValue>,
}

// What item, with what aspects, and how much of it is available.
#[derive(Clone, Debug, Component)]
pub struct InventoryItem {
    pub amount: usize,
    pub item: Item,
    pub aspects: ItemAspect,
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
