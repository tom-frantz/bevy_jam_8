use bevy::{
    camera::{Camera, ViewportConversionError},
    ecs::system::Single,
    math::Vec2,
    transform::components::GlobalTransform,
};

pub type CameraPosComponents<'a, 'b> = (&'a Camera, &'b GlobalTransform);

pub trait Camera2dPosExt {
    fn viewport_to_world_2d(&self, position: Vec2) -> Result<Vec2, ViewportConversionError>;
}

impl Camera2dPosExt for CameraPosComponents<'_, '_> {
    fn viewport_to_world_2d(&self, position: Vec2) -> Result<Vec2, ViewportConversionError> {
        let (camera, camera_transform) = self;
        camera.viewport_to_world_2d(camera_transform, position)
    }
}

impl Camera2dPosExt for Single<'_, '_, CameraPosComponents<'_, '_>> {
    fn viewport_to_world_2d(&self, position: Vec2) -> Result<Vec2, ViewportConversionError> {
        let (camera, camera_transform) = **self;
        camera.viewport_to_world_2d(camera_transform, position)
    }
}
