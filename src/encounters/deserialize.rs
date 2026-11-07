use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};

use crate::{asset_helpers::DeError, encounters::Encounter};

#[derive(Default, TypePath)]
pub struct TomlEncLoader;

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

#[cfg(test)]
mod tests {
    use std::{
        fs::{read_dir, read_to_string},
        path::Path,
    };

    use toml;

    use super::*;

    #[test]
    fn encounters_parse() {
        for entry in read_dir(Path::new("assets/encounters")).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let asset: Result<Encounter, _> = toml::from_str(&read_to_string(path).unwrap());

            if let Err(val) = &asset {
                dbg!(val);
            };
            if let Ok(val) = &asset {
                dbg!(val);
            }
            assert!(asset.is_ok())
        }
    }
}
