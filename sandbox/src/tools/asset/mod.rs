use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use json::JsonValue;
use innovus::gfx::{Image, ImageAtlas, Program, ProgramPreset, Shader, ShaderType, Texture2D, TextureSampling, TextureWrap};
use innovus::gfx::color::ColorPalette;
use innovus::tools::Rectangle;
use crate::tools::asset::block::{BlockAppearance, BlockImage};
use crate::tools::asset::entity::EntityImage;
use crate::world::block::{Block, BlockType, ChunkLocation, BLOCK_TYPES};
use crate::world::item::ItemType;
use crate::world::item::types::ITEM_TYPES;

pub mod anim;
pub mod block;
pub mod entity;

pub struct AssetPool {
    assets_path: PathBuf,
    default_shaders: Program,
    gui_texture: Texture2D,
    gui_atlas: ImageAtlas,
    gui_images: HashMap<String, Rectangle<u32>>,
    gui_shaders: Program,
    block_texture: Texture2D,
    block_atlas: ImageAtlas,
    block_appearances: HashMap<*const BlockType, BlockAppearance>,
    block_shaders: Program,
    item_texture: Texture2D,
    item_atlas: ImageAtlas,
    item_images: HashMap<*const ItemType, Rectangle<u32>>,
    entity_texture: Texture2D,
    entity_atlas: ImageAtlas,
    entity_images: HashMap<String, EntityImage>,
    font_texture: Texture2D,
    color_palettes: HashMap<String, ColorPalette>,
    text_strings: HashMap<String, String>,
}

impl AssetPool {
    pub fn load(assets_path: impl AsRef<Path>) -> Result<Self, String> {
        fn create_texture(bind_slot: u32) -> Texture2D {
            let mut texture = Texture2D::create(bind_slot);
            texture.set_minify_sampling(TextureSampling::Nearest);
            texture.set_magnify_sampling(TextureSampling::Nearest);
            texture.set_wrap_s(TextureWrap::Repeat);
            texture.set_wrap_t(TextureWrap::Repeat);
            texture
        }

        let mut assets = Self {
            assets_path: assets_path.as_ref().into(),
            default_shaders: Program::from_preset(ProgramPreset::Default2DShader)?,
            gui_texture: create_texture(0),
            gui_atlas: ImageAtlas::new(Default::default()),
            gui_images: HashMap::new(),
            gui_shaders: Program::create()?,
            block_texture: create_texture(0),
            block_atlas: ImageAtlas::new(Default::default()),
            block_appearances: HashMap::new(),
            block_shaders: Program::create()?,
            item_texture: create_texture(0),
            item_atlas: ImageAtlas::new(Default::default()),
            item_images: HashMap::new(),
            entity_texture: create_texture(0),
            entity_atlas: ImageAtlas::new(Default::default()),
            entity_images: HashMap::new(),
            font_texture: create_texture(0),
            color_palettes: HashMap::new(),
            text_strings: HashMap::new(),
        };

        // Despite the name of the method, this loads everything for the first time
        assets.reload()?;

        Ok(assets)
    }

    pub fn assets_path(&self) -> &Path {
        &self.assets_path
    }

    pub fn resolve_path(&self, sub_path: impl AsRef<Path>) -> PathBuf {
        self.assets_path.join(sub_path)
    }

    pub fn load_image(&self, sub_path: impl AsRef<Path>) -> Result<Image, String> {
        let path = self.resolve_path(sub_path).with_extension("png");
        Image::load_file(&path)
    }

    pub fn load_json(&self, sub_path: impl AsRef<Path>) -> Result<JsonValue, String> {
        let path = self.resolve_path(sub_path).with_extension("json");
        let json_raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read JSON asset at '{}': {err}", path.display()))?;
        json::parse(&json_raw)
            .map_err(|err| format!("failed to parse JSON asset at '{}': {err}", path.display()))
    }

    pub fn load_text(&self, sub_path: impl AsRef<Path>) -> Result<String, String> {
        let path = self.resolve_path(sub_path);
        std::fs::read_to_string(&path).map_err(|err| err.to_string())
    }

    pub fn reload(&mut self) -> Result<(), String> {
        self.clear_gui_images();
        self.reload_block_appearances()?;
        self.reload_item_images()?;
        self.clear_entity_images();
        self.reload_font()?;
        self.clear_color_palettes();
        self.reload_shaders()?;
        self.reload_text_strings()?;

        Ok(())
    }

    pub fn default_shaders(&self) -> &Program {
        &self.default_shaders
    }

    pub fn gui_texture(&self) -> &Texture2D {
        &self.gui_texture
    }

    pub fn gui_atlas(&self) -> &ImageAtlas {
        &self.gui_atlas
    }

    pub fn get_gui_image(&mut self, key: &str) -> Result<Rectangle<u32>, String> {
        if let Some(&atlas_region) = self.gui_images.get(key) {
            Ok(atlas_region)
        }
        else {
            let loaded_image = self.load_image(format!("images/{key}"))?;
            let atlas_offset = self.gui_atlas.add_image(&loaded_image);
            let atlas_region = Rectangle::from_size(atlas_offset, loaded_image.size());
            self.gui_texture.upload_image(self.gui_atlas.image());

            self.gui_images.insert(key.into(), atlas_region);
            Ok(atlas_region)
        }
    }

    pub fn clear_gui_images(&mut self) {
        self.gui_images.clear();
        self.gui_atlas.clear();
    }

    pub fn gui_shaders(&self) -> &Program {
        &self.gui_shaders
    }

    pub fn block_texture(&self) -> &Texture2D {
        &self.block_texture
    }

    pub fn block_atlas(&self) -> &ImageAtlas {
        &self.block_atlas
    }

    pub fn reload_block_appearances(&mut self) -> Result<(), String> {
        self.block_appearances.clear();
        self.block_atlas.clear();

        let default_block_image = {
            let metadata = self.load_json("defaults/block_image")?;
            BlockImage::parse("defaults/block_image".into(), &metadata, None)?
        };
        let default_states_data = self.load_json("defaults/block_states")?;
        let mut block_images: HashMap<String, BlockImage> = HashMap::new();

        for &block_type in BLOCK_TYPES {
            let states_data = self.load_json(format!("states/block/{block_type}"))
                .unwrap_or_else(|_| default_states_data.clone());

            let block_appearance = BlockAppearance::parse(&states_data, block_type, |image_key| {
                if let Some(block_image) = block_images.get(image_key) {
                    Ok(block_image.clone())
                }
                else {
                    let path = format!("images/{image_key}");

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

            self.block_appearances.insert(block_type, block_appearance);
        }

        self.block_texture.upload_image(self.block_atlas.image());

        Ok(())
    }

    pub fn get_block_appearance(&self, block_type: &'static BlockType) -> &BlockAppearance {
        &self.block_appearances[&(block_type as *const _)]
    }

    pub fn get_block_image(&self, block: &Block, chunk_location: ChunkLocation, x: usize, y: usize) -> Option<&BlockImage> {
        self.get_block_appearance(block.block_type()).get_image(block, chunk_location, x, y)
    }

    pub fn block_shaders(&self) -> &Program {
        &self.block_shaders
    }

    pub fn item_texture(&self) -> &Texture2D {
        &self.item_texture
    }

    pub fn item_atlas(&self) -> &ImageAtlas {
        &self.item_atlas
    }

    pub fn reload_item_images(&mut self) -> Result<(), String> {
        self.item_images.clear();
        self.item_atlas.clear();

        for &item_type in ITEM_TYPES {
            let path = format!("images/item/{item_type}");

            let Ok(loaded_image) = self.load_image(&path) else {
                continue;
            };
            let atlas_offset = self.item_atlas.add_image(&loaded_image);
            let atlas_region = Rectangle::from_size(atlas_offset, loaded_image.size());

            self.item_images.insert(item_type, atlas_region);
        }

        self.item_texture.upload_image(self.item_atlas.image());

        Ok(())
    }

    pub fn get_item_image(&self, item_type: &'static ItemType) -> Option<Rectangle<u32>> {
        self.item_images.get(&(item_type as *const _)).copied()
    }

    pub fn entity_texture(&self) -> &Texture2D {
        &self.entity_texture
    }

    pub fn entity_atlas(&self) -> &ImageAtlas {
        &self.entity_atlas
    }

    pub fn get_entity_image(&mut self, key: &str) -> Result<EntityImage, String> {
        if let Some(entity_image) = self.entity_images.get(key) {
            Ok(entity_image.clone())
        }
        else {
            let path = format!("images/{key}");

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
        let font_image = self.load_image("images/font/unicode_0")?;
        self.font_texture.upload_image(&font_image);
        Ok(())
    }

    pub fn get_color_palette(&mut self, key: &str) -> Result<&ColorPalette, String> {
        if !self.color_palettes.contains_key(key) {
            let path = self.resolve_path(format!("palettes/{key}")).with_extension("gpl");
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

    pub fn get_text_string<'a>(&'a self, key: &'a str) -> &'a str {
        match self.text_strings.get(key) {
            Some(string) => string,
            None => key,
        }
    }

    pub fn reload_text_strings(&mut self) -> Result<(), String> {
        self.text_strings.clear();

        fn parse<'a>(value: &'a JsonValue, prefix: &mut Vec<&'a str>, text_strings: &mut HashMap<String, String>) {
            for (inner_key, inner_value) in value.entries() {
                prefix.push(inner_key);
                parse(inner_value, prefix, text_strings);
                prefix.pop();
            }
            if let Some(string) = value.as_str() {
                text_strings.insert(prefix.join("."), string.into());
            }
        }

        let data = self.load_json("text/en_us")?;
        parse(&data, &mut Vec::new(), &mut self.text_strings);

        Ok(())
    }

    pub fn reload_shaders(&mut self) -> Result<(), String> {
        self.gui_shaders.attach_shader(&Shader::create(
            &self.load_text("shaders/gui_v.glsl")?,
            ShaderType::Vertex,
        )?);
        self.gui_shaders.attach_shader(&Shader::create(
            &self.load_text("shaders/gui_f.glsl")?,
            ShaderType::Fragment,
        )?);
        self.gui_shaders.link()?;

        self.block_shaders.attach_shader(&Shader::create(
            &self.load_text("shaders/block_v.glsl")?,
            ShaderType::Vertex,
        )?);
        self.block_shaders.attach_shader(&Shader::create(
            &self.load_text("shaders/block_f.glsl")?,
            ShaderType::Fragment,
        )?);
        self.block_shaders.link()?;

        Ok(())
    }
}
