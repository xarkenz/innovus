use std::collections::HashMap;
use std::path::{Path, PathBuf};
use innovus::gfx::{Image, ImageAtlas, Texture2D, TextureSampling, TextureWrap};
use innovus::tools::{Rectangle, Vector};
use crate::tools::asset::entity::EntityImage;
use crate::world::block::{BlockAppearance, BlockType};

pub mod entity;
pub mod block;

#[derive(Clone, PartialEq, Debug)]
pub struct ImageAnimation {
    pub frame_count: u32,
    pub frame_time: u32,
}

pub struct AssetPool {
    assets_path: PathBuf,
    block_texture: Texture2D,
    block_atlas: ImageAtlas,
    block_appearances: HashMap<*const BlockType, BlockAppearance>,
    entity_texture: Texture2D,
    entity_atlas: ImageAtlas,
    entity_images: HashMap<String, EntityImage>,
}

impl AssetPool {
    pub fn new(assets_path: impl AsRef<Path>) -> Result<Self, String> {
        let assets_path = assets_path.as_ref();

        let mut block_atlas = ImageAtlas::new(Default::default());
        let block_appearances = crate::world::block::BLOCK_TYPES
            .iter()
            .filter_map(|&block_type| {
                let path = assets_path.join(format!("textures/block/{}.png", block_type.name));
                Image::from_file(&path).ok().map(|image| {
                    let offset = block_atlas.add_image(&image);
                    let appearance = BlockAppearance {
                        block_type,
                        offset,
                        resolution: 16,
                    };
                    (block_type as *const BlockType, appearance)
                })
            })
            .collect();

        let mut block_texture = Texture2D::new(0);
        block_texture.set_minify_filter(TextureSampling::Nearest);
        block_texture.set_magnify_filter(TextureSampling::Nearest);
        block_texture.set_wrap_s(TextureWrap::Repeat);
        block_texture.set_wrap_t(TextureWrap::Repeat);
        block_texture.load_from_image(block_atlas.image());

        let mut entity_texture = Texture2D::new(0);
        entity_texture.set_minify_filter(TextureSampling::Nearest);
        entity_texture.set_magnify_filter(TextureSampling::Nearest);
        entity_texture.set_wrap_s(TextureWrap::Repeat);
        entity_texture.set_wrap_t(TextureWrap::Repeat);

        Ok(Self {
            assets_path: assets_path.into(),
            block_texture,
            block_atlas,
            block_appearances,
            entity_texture,
            entity_atlas: ImageAtlas::new(Default::default()),
            entity_images: HashMap::new(),
        })
    }

    pub fn assets_path(&self) -> &Path {
        &self.assets_path
    }

    pub fn get_asset_path(&self, sub_path: impl AsRef<Path>) -> PathBuf {
        self.assets_path.join(sub_path)
    }

    pub fn block_atlas(&self) -> &ImageAtlas {
        &self.block_atlas
    }

    pub fn block_texture(&self) -> &Texture2D {
        &self.block_texture
    }

    pub fn get_block_appearance(&self, block_type: &'static BlockType) -> Option<&BlockAppearance> {
        self.block_appearances.get(&(block_type as *const BlockType))
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
            let image_path = self.get_asset_path(format!("textures/entity/{key}.png"));
            let loaded_image = Image::from_file(&image_path)?;
            let atlas_origin = self.entity_atlas.add_image(&loaded_image);
            self.entity_texture.load_from_image(self.entity_atlas.image());

            let atlas_region = Rectangle::from_size(
                atlas_origin,
                Vector([loaded_image.width(), loaded_image.height()]),
            );

            let metadata_path = image_path.with_extension("json");
            let metadata_json = std::fs::read_to_string(&metadata_path)
                .map_err(|err| format!("failed to read metadata file for entity/{key}: {err}"))?;
            let metadata = json::parse(&metadata_json)
                .map_err(|err| format!("failed to parse metadata for entity/{key}: {err}"))?;

            let entity_image = EntityImage::parse(key, atlas_region, &metadata)?;
            self.entity_images.insert(key.into(), entity_image.clone());
            Ok(entity_image)
        }
    }
}
