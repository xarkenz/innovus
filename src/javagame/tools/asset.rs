use std::collections::HashMap;
use innovus::gfx::{Image, Texture2D, TextureSampling, TextureWrap};
use crate::world::block::{types::BLOCK_TYPES, BlockAppearance, BlockType};

pub struct BlockGraphics {
    atlas: Image,
    texture: Texture2D,
    appearances: HashMap<*const BlockType, BlockAppearance>,
}

impl BlockGraphics {
    pub fn new() -> Result<Self, String> {
        let (images, block_types): (Vec<Image>, Vec<&BlockType>) = BLOCK_TYPES.iter()
            .filter_map(|&block_type| {
                let path = format!("src/javagame/assets/textures/block/{}.png", block_type.name);
                Image::from_file(&path).ok().map(|image| (image, block_type))
            })
            .unzip();

        let (atlas, offsets) = Image::new_atlas(&images);

        let appearances = HashMap::from_iter(block_types.iter()
            .zip(offsets)
            .map(|(&block_type, offset)| {
                let appearance = BlockAppearance {
                    block_type,
                    offset,
                    resolution: 16,
                };
                (block_type as *const BlockType, appearance)
            }));

        let mut texture = Texture2D::new(0);
        texture.set_minify_filter(TextureSampling::Nearest);
        texture.set_magnify_filter(TextureSampling::Nearest);
        texture.set_wrap_s(TextureWrap::Repeat);
        texture.set_wrap_t(TextureWrap::Repeat);
        texture.load_from_image(&atlas);

        Ok(Self {
            atlas,
            texture,
            appearances,
        })
    }

    pub fn atlas(&self) -> &Image {
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
    //
}
