use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};

use crate::{asset_helpers::DeError, encounters::EncounterRes};

#[derive(Default, TypePath)]
pub struct TomlEncLoader;

impl AssetLoader for TomlEncLoader {
    type Asset = EncounterRes;
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

        let enc: EncounterRes = toml::from_slice(&bytes)?;
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
        let mut failures = Vec::new();
        let mut dirs = vec![Path::new("assets/encounters").to_path_buf()];
        while let Some(dir) = dirs.pop() {
            for entry in read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                    continue;
                }
                let asset: Result<EncounterRes, _> =
                    toml::from_str(&read_to_string(&path).unwrap());
                if let Err(err) = asset {
                    failures.push(format!("{}:\n{err}", path.display()));
                }
            }
        }
        assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    }
}
