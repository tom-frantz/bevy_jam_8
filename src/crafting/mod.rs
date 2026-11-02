use bevy::{
    app::Plugin,
    ecs::{component::Component, hierarchy::Children, system::Commands},
    scene::{CommandsSceneExt, bsn},
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    ui::{Node, Val, widget::Text},
};

use crate::InGameScreen;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::Crafting), enter_crafting_screen);
    }
}

/// When moving to the crafting screen, run this system once to setup everything.
fn enter_crafting_screen(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.)
        }
        Children [
            Text("Hello, Crafting Screen")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::Crafting)
    });
}

#[derive(Clone, Debug)]
enum Item {
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
}

#[derive(Clone, Debug, Component)]
struct InventoryItem {
    amount: usize,
    item: Item,
}
