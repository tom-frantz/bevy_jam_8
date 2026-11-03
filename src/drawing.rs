use std::time::Duration;

use bevy::{
    app::{Startup, Update},
    camera::Camera,
    color::{Color, Hue},
    curve::{CubicCardinalSpline, CubicGenerator},
    ecs::{
        observer::On,
        resource::Resource,
        system::{Local, Res, ResMut, Single},
    },
    gizmos::{
        config::{DefaultGizmoConfigGroup, GizmoConfigStore},
        gizmos::Gizmos,
    },
    log::debug,
    math::Vec2,
    picking::events::PointerDrag,
    time::Time,
    transform::components::GlobalTransform,
};

pub struct DrawingPlugin;
impl bevy::app::Plugin for DrawingPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            // .init_resource::<DrawingPath>()
            .add_observer(on_drag)
            .add_systems(Startup, setup_gizmos)
            .add_systems(Update, draw_curve);
    }
}

#[derive(Debug, Default, Resource)]
pub struct DrawingPath {
    path: Vec<Vec2>,
    current_control_point: Option<Vec2>,
}

impl DrawingPath {
    fn add_point(&mut self, point: Vec2) {
        if self.path.is_empty() {
            self.path.push(point);
            self.current_control_point = Some(point);
            return;
        }

        // Get other things for calculation
        let previous_control_point = self
            .current_control_point
            .expect("Should be set before we ever reach this point of the function");
        let last_fixed_point = self
            .path
            .last()
            .expect("should be set before ever reaching this point");
        let current_control_point = point;

        // Update the new point
        self.current_control_point = Some(current_control_point);

        // If we move too far, push the old point to the fixed points, and add the new point as the control point.
        let distance = current_control_point.distance(*last_fixed_point);
        const CUTOFF: f32 = 40.;

        if distance > CUTOFF {
            self.path.push(previous_control_point);
        }
    }
}

impl DrawingPath {
    pub fn new() -> Self {
        Self::default()
    }
}

fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.width = 15.0;
    config.line.joints = bevy::gizmos::config::GizmoLineJoint::Round(4);
}

fn on_drag(
    event: On<PointerDrag>,
    drawing_state: Option<ResMut<DrawingPath>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    // It's the crafting screens responsibility to add and remove the resource as necessary.
    let Some(mut drawing_state) = drawing_state else {
        return;
    };

    let (camera, camera_transform) = camera.into_inner();
    let pointer_position = camera
        .viewport_to_world_2d(camera_transform, event.pointer.position)
        .expect("Must work please");

    drawing_state.add_point(pointer_position);

    debug!("pointer drag event global.")
}

fn draw_curve(
    mut tracked_time: Local<Duration>,
    time: Res<Time>,
    drawing_path: Option<Res<DrawingPath>>,
    mut gizmos: Gizmos,
) {
    let Some(drawing_path) = drawing_path else {
        *tracked_time = Duration::default();
        return;
    };

    *tracked_time += time.delta();

    let mut path = drawing_path.path.clone();
    path.push(drawing_path.current_control_point.unwrap());

    let spline = CubicCardinalSpline::new_catmull_rom(path)
        .to_curve()
        .expect("Could not create curve");
    // Scale resolution with curve length so it doesn't degrade as the length increases.
    let resolution = 10 * spline.segments().len();

    const DEGREES_PER_SEC: f32 = 60.;
    let colour =
        Color::srgb(1.0, 0.0, 0.0).rotate_hue(DEGREES_PER_SEC * tracked_time.as_secs_f32());

    gizmos.linestrip(
        spline.iter_positions(resolution).map(|pt| pt.extend(0.0)),
        colour,
    );
}
