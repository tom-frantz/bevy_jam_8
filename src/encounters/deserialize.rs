use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};
use thiserror::Error;

use crate::encounters::Encounter;

#[derive(Default, TypePath)]
pub struct TomlEncLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse toml: {0}")]
    FailedToParseToml(#[from] toml::de::Error),
}

impl AssetLoader for TomlEncLoader {
    type Asset = Encounter;
    type Settings = ();
    type Error = DeError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let enc: Encounter = toml::from_slice(&bytes)?;
        Ok(enc)
    }

    fn extensions(&self) -> &[&str] {
        &["enc.toml"] // Handle files ending in .enc.toml
    }
}
