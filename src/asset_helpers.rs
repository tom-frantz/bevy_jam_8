use bevy::{
    app::{App, PluginGroup},
    asset::load_internal_binary_asset,
    image::ImagePlugin,
    text::{Font, TextFont},
};
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse toml: {0}")]
    FailedToParseToml(#[from] toml::de::Error),
}

pub trait DefaultPluginImageExt {
    fn set_image_plugin(self) -> impl PluginGroup;
}

impl<T: PluginGroup> DefaultPluginImageExt for T {
    fn set_image_plugin(self) -> impl PluginGroup {
        self.set(ImagePlugin::default_nearest())
    }
}

pub fn set_default_font(app: &mut App) {
    let bevy::text::FontSource::Handle(asset_handle) = TextFont::default().font else {
        panic!("Default font is not a handle!")
    };

    load_internal_binary_asset!(
        app,
        asset_handle,
        "../assets/PixelParchment.ttf",
        |bytes: &[u8], _path: String| { Font::from_bytes(bytes.to_vec()) }
    );
}
