use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};
use serde::Deserialize;

use crate::{
    asset_helpers::DeError,
    crafting::{Aspects, CraftingState, Item, ItemAspect},
};

#[derive(Debug, Clone, Asset, TypePath, Deserialize)]
pub struct Ingredients {
    pub ingredients: Vec<Ingredient>,
}

#[derive(Debug, Clone, Asset, TypePath, Deserialize)]
pub struct Recipes {
    pub recipes: Vec<Recipe>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ingredient {
    pub name: Item,
    pub description: String,
    pub aspects: ItemAspect,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub input_aspects: Vec<Aspects>,
    pub output: Item,
}

impl Recipe {
    /// Tests by checking if each and every aspect in the recipe is met by some
    /// ingredient. Each ingredient can only do one thing. Checks primary first,
    /// and doesn't check next aspects if already met. If an earlier item meets
    /// the item's primary aspect, then it checks the secondary, etc.
    pub fn test_for_ingredients(&self, ingredients: Vec<Ingredient>) -> bool {
        let mut aspects_remaining = self.input_aspects.clone();

        for ingredient in ingredients {
            for aspect in ingredient.aspects.aspects_in_order() {
                if let Some(index) = aspects_remaining.iter().position(|item| *item == aspect) {
                    aspects_remaining.remove(index);
                }
            }
        }

        // Checks that everything has been satisfied if nothing remains
        aspects_remaining.is_empty()
    }
}

#[derive(Default, TypePath)]
pub struct IngredientAssetLoader;
#[derive(Default, TypePath)]
pub struct RecipeAssetLoader;

impl AssetLoader for IngredientAssetLoader {
    type Asset = Ingredients;
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

        let ing: Ingredients = toml::from_slice(&bytes)?;
        Ok(ing)
    }

    fn extensions(&self) -> &[&str] {
        &["ing.toml"] // Handle files ending in .ing.toml
    }
}

impl AssetLoader for RecipeAssetLoader {
    type Asset = Recipes;
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

        let rec: Recipes = toml::from_slice(&bytes)?;
        Ok(rec)
    }

    fn extensions(&self) -> &[&str] {
        &["rec.toml"] // Handle files ending in .rec.toml
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
    fn ingredients_parse() {
        let asset: Result<Ingredients, _> =
            toml::from_str(include_str!("../../assets/ingredients.ing.toml"));

        if let Err(val) = &asset {
            dbg!(val);
        };
        if let Ok(val) = &asset {
            dbg!(val);
        }
        assert!(asset.is_ok())
    }

    #[test]
    fn recipes_parse() {
        let asset: Result<Recipes, _> =
            toml::from_str(include_str!("../../assets/recipes.rec.toml"));

        if let Err(val) = &asset {
            dbg!(val);
        };
        if let Ok(val) = &asset {
            dbg!(val);
        }
        assert!(asset.is_ok())
    }
}
