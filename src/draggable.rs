use bevy::{
    camera::Camera,
    ecs::{
        event::EntityEvent,
        observer::On,
        system::{Query, Single},
    },
    picking::{Pickable, events::PointerDrag},
    scene::{Scene, SceneComponent, bsn, on},
    transform::components::{GlobalTransform, Transform},
};

pub struct DraggablePlugin;
impl bevy::app::Plugin for DraggablePlugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}

#[derive(SceneComponent, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Draggable;
impl Draggable {
    fn scene() -> impl Scene {
        bsn! {
            Pickable
            Transform
            on(draggable_click)
        }
    }
}

pub fn draggable_click(
    mut event: On<PointerDrag>,
    mut draggables: Query<&mut Transform>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let mut transform = draggables
        .get_mut(event.event_target())
        .expect("A draggable must have transform");

    let (camera, c_global_transform) = camera.into_inner();
    let pointer_pos = event.pointer.position;
    transform.translation = camera
        .viewport_to_world_2d(c_global_transform, pointer_pos)
        .expect("Should be able to get the world position")
        .extend(transform.translation.z);
    event.propagate(false);
}
