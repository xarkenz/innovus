use std::collections::HashMap;
use innovus::gfx::{Image, ImageAtlas, Texture2D, TextureSampling, TextureWrap};
use crate::world::block::{types::BLOCK_TYPES, BlockAppearance, BlockType};

pub struct BlockGraphics {
    atlas: ImageAtlas,
    texture: Texture2D,
    appearances: HashMap<*const BlockType, BlockAppearance>,
}

impl BlockGraphics {
    pub fn new() -> Result<Self, String> {
        let mut atlas = ImageAtlas::new(Default::default());
        let appearances = BLOCK_TYPES
            .iter()
            .filter_map(|&block_type| {
                let path = format!("src/javagame/assets/textures/block/{}.png", block_type.name);
                Image::from_file(&path).ok().map(|image| {
                    let offset = atlas.add_image(&image);
                    let appearance = BlockAppearance {
                        block_type,
                        offset,
                        resolution: 16,
                    };
                    (block_type as *const BlockType, appearance)
                })
            })
            .collect();

        let mut texture = Texture2D::new(0);
        texture.set_minify_filter(TextureSampling::Nearest);
        texture.set_magnify_filter(TextureSampling::Nearest);
        texture.set_wrap_s(TextureWrap::Repeat);
        texture.set_wrap_t(TextureWrap::Repeat);
        texture.load_from_image(atlas.image());

        Ok(Self {
            atlas,
            texture,
            appearances,
        })
    }

    pub fn atlas(&self) -> &ImageAtlas {
        &self.atlas
    }

    pub fn texture(&self) -> &Texture2D {
        &self.texture
    }

    pub fn get_appearance(&self, block_type: &'static BlockType) -> Option<&BlockAppearance> {
        self.appearances.get(&(block_type as *const BlockType))
    }
}

pub struct EntityGraphics {
    atlas: ImageAtlas,
    texture: Texture2D,
}

impl EntityGraphics {
    pub fn new() -> Result<Self, String> {
        let mut texture = Texture2D::new(1);
        texture.set_minify_filter(TextureSampling::Nearest);
        texture.set_magnify_filter(TextureSampling::Nearest);
        texture.set_wrap_s(TextureWrap::Repeat);
        texture.set_wrap_t(TextureWrap::Repeat);

        Ok(Self {
            atlas: ImageAtlas::new(Default::default()),
            texture,
        })
    }
}
