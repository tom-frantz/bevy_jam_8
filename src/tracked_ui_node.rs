use bevy::{
    app::Update,
    camera::Camera,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Query, Single},
    },
    transform::components::GlobalTransform,
    ui::{Node, Val::Px},
};

pub struct TrackedUiNodePlugin;
impl bevy::app::Plugin for TrackedUiNodePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, sync_transform_propogated_ui_nodes);
    }
}

// This is the relationship target on the parent
#[derive(Component)]
#[relationship_target(relationship = TrackedTransform, linked_spawn)]
pub struct TransformPropogatedUiNode(Vec<Entity>);

// This is the relationship FROM child TO parent
#[derive(Component)]
#[relationship(relationship_target = TransformPropogatedUiNode)]
pub struct TrackedTransform(Entity);

fn sync_transform_propogated_ui_nodes(
    mut pages: Query<(&mut Node, &TrackedTransform)>,
    parents: Query<&GlobalTransform>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera.into_inner();
    for (mut node, parent_page) in &mut pages {
        let parent = parents
            .get(parent_page.0)
            .expect("Parent must have a global transform");

        let viewport_position = camera
            .world_to_viewport(camera_transform, parent.translation())
            .expect("Must work please");

        node.left = Px(viewport_position.x);
        node.top = Px(viewport_position.y);
    }
}
