use bevy::{
    app::{App, Plugin, Update},
    color::{Alpha, Color},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Res, ResMut, Single},
    },
    state::state::NextState,
    time::{Time, Timer, TimerMode},
    ui::{BackgroundColor, GlobalZIndex, Node, PositionType, Val},
    utils::default,
};

use crate::{InGameScreen, PlayerLocation};

const FADE_SECONDS: f32 = 0.6;

pub struct FaderPlugin;

impl Plugin for FaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fader>();
        app.add_systems(Update, (start_fade, tick_fade).chain().run_if(fader_active));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FadePhase {
    #[default]
    Idle,
    Starting,
    ToBlack,
    FromBlack,
}

#[derive(Resource)]
pub struct Fader {
    timer: Timer,
    phase: FadePhase,
    pending: Option<(PlayerLocation, InGameScreen)>,
}

impl Default for Fader {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(FADE_SECONDS, TimerMode::Once),
            phase: FadePhase::Idle,
            pending: None,
        }
    }
}

impl Fader {
    // fade in and out
    pub fn fade_to(&mut self, location: PlayerLocation, screen: InGameScreen) {
        if self.is_fading() {
            return;
        }
        self.pending = Some((location, screen));
        self.phase = FadePhase::Starting;
        self.timer.reset();
    }

    pub fn is_fading(&self) -> bool {
        self.phase != FadePhase::Idle
    }
}

#[derive(Component)]
struct FadeOverlay;

fn fader_active(fader: Res<Fader>) -> bool {
    fader.is_fading()
}

fn start_fade(mut commands: Commands, mut fader: ResMut<Fader>) {
    if fader.phase != FadePhase::Starting {
        return;
    }
    commands.spawn((
        FadeOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.)),
        GlobalZIndex(50),
    ));
    fader.phase = FadePhase::ToBlack;
}

fn tick_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut fader: ResMut<Fader>,
    overlay: Single<(Entity, &mut BackgroundColor), With<FadeOverlay>>,
    mut next_location: ResMut<NextState<PlayerLocation>>,
    mut next_screen: ResMut<NextState<InGameScreen>>,
) {
    let (entity, mut color) = overlay.into_inner();
    fader.timer.tick(time.delta());
    let t = fader.timer.fraction();

    match fader.phase {
        FadePhase::ToBlack => {
            color.0.set_alpha(t);
            if fader.timer.is_finished() {
                if let Some((location, screen)) = fader.pending.take() {
                    next_location.set(location);
                    next_screen.set(screen);
                }
                fader.phase = FadePhase::FromBlack;
                fader.timer.reset();
            }
        }
        FadePhase::FromBlack => {
            color.0.set_alpha(1. - t);
            if fader.timer.is_finished() {
                commands.entity(entity).despawn();
                fader.phase = FadePhase::Idle;
            }
        }
        FadePhase::Idle | FadePhase::Starting => {}
    }
}
