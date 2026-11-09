use bevy::{
    app::Plugin,
    color::Color,
    ecs::{
        component::Component,
        event::EntityEvent,
        hierarchy::Children,
        observer::On,
        resource::Resource,
        system::{Commands, Query, ResMut, command::insert_resource},
    },
    log::info,
    math::Vec3,
    picking::{
        Pickable,
        events::{Pointer, PointerPress},
    },
    scene::{CommandsSceneExt, Scene, bsn, bsn_list, on},
    sprite::Sprite,
    state::{
        state::{NextState, OnEnter, OnExit},
        state_scoped::DespawnOnExit,
    },
    transform::components::Transform,
    ui::{
        GlobalZIndex, Node, Val, ZIndex,
        widget::{ImageNode, NodeImageMode, Text},
    },
};
use serde::Deserialize;

use crate::{AppState, InGameScreen, PlayerLocation};

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(AppState::InGame), |mut cmds: Commands| {
            cmds.insert_resource(BroomFuel::default())
        })
        .add_systems(OnEnter(InGameScreen::WorldMap), enter_world_map)
        .add_systems(OnExit(PlayerLocation::Home), top_up_broom_fuel);
    }
}

#[derive(Debug, Resource)]
pub struct BroomFuel {
    distance: usize,
}

/// Once the night is over, the broom should be back in action c:
fn top_up_broom_fuel() {
    // TODO actually top up broom fuel
}

impl Default for BroomFuel {
    fn default() -> Self {
        Self { distance: 10 }
    }
}

/// When moving to the world map, run this system once to setup everything.
///
/// This is the entrypoint as well, so have to consider it's not doint something idk
fn enter_world_map(mut commands: Commands) {
    info!("Entering world map");
    commands.spawn_scene_list(bsn_list! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.)
        }
        Sprite{
            image: "backgrounds/backgrounds_blue.png",
        }
        Transform::from_xyz(0.0, 0.0, -1.0)
        Children [
            Text("Hello, WorldMap")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::WorldMap)
        Pickable::IGNORE
        --
        @island()
        Islands::TREE
        Transform {
            translation: Vec3::new(50., 100., 0.)
        }
        --
        @island()
        Islands::MOON
        Transform {
            translation: Vec3::new(300., 0., 0.)
        }
        --
        @island()
        Islands::RUINS
        Transform {
            translation: Vec3::new(-200., -100., 0.)
        }
        --
        @island()
        Islands::PARTY
        Transform {
            translation: Vec3::new(-400., 50., 0.)
        }

    });
}

fn island() -> impl Scene {
    bsn! {
        DespawnOnExit<InGameScreen>(InGameScreen::WorldMap)
        Transform
        Sprite { image: "floating_island.png" }
        Pickable
        on(island_click_observer)
    }
}

fn island_click_observer(
    event: On<PointerPress>,
    islands: Query<&Islands>,
    mut next_state: ResMut<NextState<PlayerLocation>>,
    mut next_screen: ResMut<NextState<InGameScreen>>,
) {
    info!("This is getting pretty frisky!");
    let island = islands
        .get(event.event_target())
        .expect("Clicked on an island without an island ID");

    // TODO broom fuel consumption
    next_state.set(PlayerLocation::Island(*island));
    next_screen.set(InGameScreen::Encounter);
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Islands {
    #[default]
    TUTORIAL,
    NIGHT,
    TREE,
    MOON,
    RUINS,
    PARTY,
    ENDING,
}

impl Islands {
    pub fn background_path(self) -> String {
        match self {
            Islands::TUTORIAL => String::from("backgrounds/background_night.png"),
            Islands::NIGHT => String::from("backgrounds/background_night.png"),
            Islands::TREE => String::from("backgrounds/backgrounds_green_2.png"),
            Islands::MOON => String::from("backgrounds/backgrounds_blue_2.png"),
            Islands::RUINS => String::from("backgrounds/backgrounds_green.png"),
            Islands::PARTY => String::from("backgrounds/backgrounds_yellow.png"),
            Islands::ENDING => String::from("backgrounds/background_night.png"),
        }
    }
}
