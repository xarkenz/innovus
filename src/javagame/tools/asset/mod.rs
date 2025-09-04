use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use json::JsonValue;
use innovus::gfx::{Image, ImageAtlas, Texture2D, TextureSampling, TextureWrap};
use innovus::gfx::color::ColorPalette;
use innovus::tools::Rectangle;
use crate::tools::asset::block::{BlockAppearance, BlockImage};
use crate::tools::asset::entity::EntityImage;
use crate::world::block::{Block, BlockType, ChunkLocation, BLOCK_TYPES};

pub mod entity;
pub mod block;
pub mod anim;

pub struct AssetPool {
    assets_path: PathBuf,
    block_texture: Texture2D,
    block_atlas: ImageAtlas,
    block_appearances: HashMap<*const BlockType, BlockAppearance>,
    entity_texture: Texture2D,
    entity_atlas: ImageAtlas,
    entity_images: HashMap<String, EntityImage>,
    font_texture: Texture2D,
    color_palettes: HashMap<String, ColorPalette>,
}

impl AssetPool {
    pub fn load(assets_path: impl AsRef<Path>) -> Result<Self, String> {
        let assets_path = assets_path.as_ref();

        let mut block_texture = Texture2D::new(0);
        block_texture.set_minify_filter(TextureSampling::Nearest);
        block_texture.set_magnify_filter(TextureSampling::Nearest);
        block_texture.set_wrap_s(TextureWrap::Repeat);
        block_texture.set_wrap_t(TextureWrap::Repeat);

        let mut entity_texture = Texture2D::new(0);
        entity_texture.set_minify_filter(TextureSampling::Nearest);
        entity_texture.set_magnify_filter(TextureSampling::Nearest);
        entity_texture.set_wrap_s(TextureWrap::Repeat);
        entity_texture.set_wrap_t(TextureWrap::Repeat);

        let mut font_texture = Texture2D::new(0);
        font_texture.set_minify_filter(TextureSampling::Nearest);
        font_texture.set_magnify_filter(TextureSampling::Nearest);
        font_texture.set_wrap_s(TextureWrap::Repeat);
        font_texture.set_wrap_t(TextureWrap::Repeat);

        let mut assets = Self {
            assets_path: assets_path.into(),
            block_texture,
            block_atlas: ImageAtlas::new(Default::default()),
            block_appearances: HashMap::new(),
            entity_texture,
            entity_atlas: ImageAtlas::new(Default::default()),
            entity_images: HashMap::new(),
            font_texture,
            color_palettes: HashMap::new(),
        };

        // Despite the name of the method, this loads block appearances for the first time
        assets.reload_block_appearances()?;
        assets.reload_font()?;

        Ok(assets)
    }

    pub fn assets_path(&self) -> &Path {
        &self.assets_path
    }

    pub fn get_asset_path(&self, sub_path: impl AsRef<Path>) -> PathBuf {
        self.assets_path.join(sub_path)
    }

    pub fn load_image(&self, path: impl AsRef<Path>) -> Result<Image, String> {
        let path = path.as_ref().with_extension("png");
        Image::load_file(&path)
    }

    pub fn load_json(&self, path: impl AsRef<Path>) -> Result<JsonValue, String> {
        let path = path.as_ref().with_extension("json");
        let json_raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read JSON asset at '{}': {err}", path.display()))?;
        json::parse(&json_raw)
            .map_err(|err| format!("failed to parse JSON asset at '{}': {err}", path.display()))
    }

    pub fn reload(&mut self) -> Result<(), String> {
        self.reload_block_appearances()?;
        self.clear_entity_images();
        self.reload_font()?;
        self.clear_color_palettes();
        Ok(())
    }

    pub fn block_atlas(&self) -> &ImageAtlas {
        &self.block_atlas
    }

    pub fn block_texture(&self) -> &Texture2D {
        &self.block_texture
    }

    pub fn reload_block_appearances(&mut self) -> Result<(), String> {
        self.block_appearances.clear();
        self.block_atlas.clear();

        let default_block_image = {
            let path = self.get_asset_path("defaults/block_image");
            let metadata = self.load_json(&path)?;
            BlockImage::parse("defaults/block_image".into(), &metadata, None)?
        };
        let default_states_data = {
            let path = self.get_asset_path("defaults/block_states");
            self.load_json(&path)?
        };
        let mut block_images: HashMap<String, BlockImage> = HashMap::new();

        for &block_type in BLOCK_TYPES {
            let states_path = self.get_asset_path(format!("states/block/{}", block_type.name));
            let states_data = self.load_json(states_path).unwrap_or_else(|_| default_states_data.clone());

            let block_appearance = BlockAppearance::parse(&states_data, block_type, |image_key| {
                if let Some(block_image) = block_images.get(image_key) {
                    Ok(block_image.clone())
                }
                else {
                    let path = self.get_asset_path(format!("images/{image_key}"));

                    let loaded_image = self.load_image(&path)?;
                    let atlas_offset = self.block_atlas.add_image(&loaded_image);
                    let atlas_region = Rectangle::from_size(atlas_offset, loaded_image.size());

                    let mut block_image;
                    if let Ok(metadata) = self.load_json(&path) {
                        block_image = BlockImage::parse(image_key.into(), &metadata, Some(&default_block_image))?;
                    }
                    else {
                        block_image = default_block_image.clone();
                        block_image.set_key(image_key.into());
                    }
                    block_image.set_atlas_region(atlas_region)?;

                    block_images.insert(image_key.into(), block_image.clone());
                    Ok(block_image)
                }
            })?;

            self.block_appearances.insert(block_type as *const _, block_appearance);
        }

        self.block_texture.upload_image(self.block_atlas.image());

        Ok(())
    }

    pub fn get_block_appearance(&self, block_type: &'static BlockType) -> &BlockAppearance {
        &self.block_appearances[&(block_type as *const BlockType)]
    }

    pub fn get_block_image(&self, block: &Block, chunk_location: ChunkLocation, x: usize, y: usize) -> Option<&BlockImage> {
        self.get_block_appearance(block.block_type()).get_image(block, chunk_location, x, y)
    }

    pub fn entity_atlas(&self) -> &ImageAtlas {
        &self.entity_atlas
    }

    pub fn entity_texture(&self) -> &Texture2D {
        &self.entity_texture
    }

    pub fn get_entity_image(&mut self, key: &str) -> Result<EntityImage, String> {
        if let Some(entity_image) = self.entity_images.get(key) {
            Ok(entity_image.clone())
        }
        else {
            let path = self.get_asset_path(format!("images/{key}"));

            let loaded_image = self.load_image(&path)?;
            let atlas_offset = self.entity_atlas.add_image(&loaded_image);
            let atlas_region = Rectangle::from_size(atlas_offset, loaded_image.size());
            self.entity_texture.upload_image(self.entity_atlas.image());

            let metadata = self.load_json(&path)?;

            let entity_image = EntityImage::parse(key, atlas_region, &metadata)?;
            self.entity_images.insert(key.into(), entity_image.clone());
            Ok(entity_image)
        }
    }

    pub fn clear_entity_images(&mut self) {
        self.entity_images.clear();
        self.entity_atlas.clear();
    }

    pub fn font_texture(&self) -> &Texture2D {
        &self.font_texture
    }

    pub fn reload_font(&mut self) -> Result<(), String> {
        let font_image = self.load_image(self.get_asset_path("images/font/unicode_0"))?;
        self.font_texture.upload_image(&font_image);
        Ok(())
    }

    pub fn get_color_palette(&mut self, key: &str) -> Result<&ColorPalette, String> {
        if !self.color_palettes.contains_key(key) {
            let path = self.get_asset_path(format!("palettes/{key}")).with_extension("gpl");
            let palette_file = File::open(&path)
                .map_err(|err| format!("failed to read color palette at '{}': {err}", path.display()))?;
            let color_palette = ColorPalette::parse_gpl(palette_file)?;
            self.color_palettes.insert(key.into(), color_palette);
        }

        Ok(&self.color_palettes[key])
    }

    pub fn clear_color_palettes(&mut self) {
        self.color_palettes.clear();
    }
}
