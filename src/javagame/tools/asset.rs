use std::collections::HashMap;
use innovus::gfx::{Image, ImageAtlas, Texture2D, TextureSampling, TextureWrap};
use innovus::tools::{Rectangle, Vector};
use crate::world::block::{types::BLOCK_TYPES, BlockAppearance, BlockType};

pub struct AssetPool {
    block_texture: Texture2D,
    block_atlas: ImageAtlas,
    block_appearances: HashMap<*const BlockType, BlockAppearance>,
    entity_texture: Texture2D,
    entity_atlas: ImageAtlas,
    entity_image_uv: HashMap<String, Rectangle<u32>>,
}

impl AssetPool {
    pub fn new() -> Result<Self, String> {
        let mut block_atlas = ImageAtlas::new(Default::default());
        let block_appearances = BLOCK_TYPES
            .iter()
            .filter_map(|&block_type| {
                let path = format!("src/javagame/assets/textures/block/{}.png", block_type.name);
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
            block_texture,
            block_atlas,
            block_appearances,
            entity_texture,
            entity_atlas: ImageAtlas::new(Default::default()),
            entity_image_uv: HashMap::new(),
        })
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

    pub fn get_entity_image_uv(&mut self, key: &str) -> Result<Rectangle<u32>, String> {
        if let Some(&uv) = self.entity_image_uv.get(key) {
            Ok(uv)
        }
        else {
            let path = format!("src/javagame/assets/textures/entity/{key}.png");
            let loaded_image = Image::from_file(&path)?;
            let uv_origin = self.entity_atlas.add_image(&loaded_image);
            self.entity_texture.load_from_image(self.entity_atlas.image());

            let uv = Rectangle::from_size(
                uv_origin,
                Vector([loaded_image.width(), loaded_image.height()]),
            );
            self.entity_image_uv.insert(key.into(), uv);
            Ok(uv)
        }
    }
}
