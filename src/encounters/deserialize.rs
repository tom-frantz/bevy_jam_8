use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct Encounter {
    pub name: String,
    pub context_items: Vec<Context>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Context {
    pub name: String,
    pub text: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub text: String,
    pub goto_context: Option<String>, // TODO how to map this to a context.name?
    pub requires: Option<Vec<Item>>,
    pub execute: Option<Vec<Execute>>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Execute {
    AddItem(Item),
    LoseItem(Item),
    Leave,
}

// TODO Placeholder will be the broader item system
#[derive(Debug, Deserialize, Clone)]
pub enum Item {
    Mushroom,
    Stone,
}

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
