use bevy::{
    app::Plugin,
    ecs::{hierarchy::Children, observer::On, system::Commands},
    log::info,
    math::Vec3,
    picking::{
        Pickable,
        events::{Pointer, PointerPress},
    },
    scene::{CommandsSceneExt, Scene, bsn, bsn_list, on},
    sprite::Sprite,
    state::{state::OnEnter, state_scoped::DespawnOnExit},
    transform::components::Transform,
    ui::{Node, Val, widget::Text},
};

use crate::InGameScreen;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(InGameScreen::WorldMap), enter_world_map);
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
        Children [
            Text("Hello, WorldMap")
        ]
        DespawnOnExit<InGameScreen>(InGameScreen::WorldMap)
        Pickable::IGNORE
        --
        @island()
        Transform {
            translation: Vec3::new(50., 100., 0.)
        }
        --
        @island()
        Transform {
            translation: Vec3::new(300., 0., 0.)
        }
        --
        @island()
        Transform {
            translation: Vec3::new(-200., -100., 0.)
        }

    });
}

fn island() -> impl Scene {
    bsn! {
        Transform
        Sprite { image: "floating_island.png" }
        Pickable
        on(island_click_observer)
    }
}

fn island_click_observer(_event: On<PointerPress>) {
    info!("This is getting pretty frisky!")
}
