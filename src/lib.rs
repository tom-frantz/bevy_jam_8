use bevy::{
    DefaultPlugins,
    app::{App, Plugin},
};

pub struct JamPlugin;

impl Plugin for JamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
    }
}
