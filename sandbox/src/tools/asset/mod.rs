use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use json::JsonValue;
use innovus::gfx::color::ColorPalette;
use innovus::gfx::Gfx;
use innovus::gfx::image::{Image, ImageAtlas};
use innovus::gfx::mesh::{Vertex, Vertex2D};
use innovus::gfx::pipeline::BindGroup;
use innovus::gfx::texture::DynamicTexture2D;
use innovus::tools::Rectangle;
use crate::gui::render::{GuiParams, GuiVertex};
use crate::tools::asset::block::{BlockAppearance, BlockImage};
use crate::tools::asset::entity::EntityImage;
use crate::world::block::{Block, BlockType, ChunkLocation, BLOCK_TYPES};
use crate::world::camera::Camera;
use crate::world::item::ItemType;
use crate::world::item::types::ITEM_TYPES;

pub mod anim;
pub mod block;
pub mod entity;

pub struct AssetPool {
    assets_path: PathBuf,
    default_pipeline: Option<wgpu::RenderPipeline>,
    gui_texture: DynamicTexture2D,
    gui_atlas: ImageAtlas,
    gui_images: HashMap<String, Rectangle<u32>>,
    gui_pipeline: Option<wgpu::RenderPipeline>,
    block_texture: DynamicTexture2D,
    block_atlas: ImageAtlas,
    block_appearances: HashMap<*const BlockType, BlockAppearance>,
    block_pipeline: Option<wgpu::RenderPipeline>,
    item_texture: DynamicTexture2D,
    item_atlas: ImageAtlas,
    item_images: HashMap<*const ItemType, Rectangle<u32>>,
    entity_texture: DynamicTexture2D,
    entity_atlas: ImageAtlas,
    entity_images: HashMap<String, EntityImage>,
    color_palettes: HashMap<String, ColorPalette>,
    text_strings: HashMap<String, String>,
}

impl AssetPool {
    pub fn load(gfx: &Gfx, assets_path: impl AsRef<Path>) -> Result<Self, String> {
        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let texture_layout = DynamicTexture2D::create_layout(gfx.device());

        let create_texture = |label| DynamicTexture2D::create(
            gfx,
            Some(label),
            sampler.clone(),
            texture_layout.clone(),
            &Image::empty(),
        );

        let mut assets = Self {
            assets_path: assets_path.as_ref().into(),
            default_pipeline: None,
            gui_texture: create_texture("GUI Texture"),
            gui_atlas: ImageAtlas::new(Default::default()),
            gui_images: HashMap::new(),
            gui_pipeline: None,
            block_texture: create_texture("Block Texture"),
            block_atlas: ImageAtlas::new(Default::default()),
            block_appearances: HashMap::new(),
            block_pipeline: None,
            item_texture: create_texture("Item Texture"),
            item_atlas: ImageAtlas::new(Default::default()),
            item_images: HashMap::new(),
            entity_texture: create_texture("Entity Texture"),
            entity_atlas: ImageAtlas::new(Default::default()),
            entity_images: HashMap::new(),
            color_palettes: HashMap::new(),
            text_strings: HashMap::new(),
        };

        // Despite the name of the method, this loads everything for the first time
        assets.reload(gfx)?;

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
        Image::from_file(&path).map_err(|err| err.to_string())
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

    pub fn reload(&mut self, gfx: &Gfx) -> Result<(), String> {
        self.clear_gui_images();
        self.reload_block_appearances()?;
        self.reload_item_images()?;
        self.clear_entity_images();
        self.clear_color_palettes();
        self.reload_pipelines(gfx)?;
        self.reload_text_strings()?;

        Ok(())
    }

    pub fn default_pipeline(&self) -> &wgpu::RenderPipeline {
        self.default_pipeline.as_ref().unwrap()
    }

    pub fn gui_texture(&self) -> &DynamicTexture2D {
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
            let atlas_region = Rectangle::from_span(atlas_offset, loaded_image.size());
            self.gui_texture.rewrite(self.gui_atlas.image());

            self.gui_images.insert(key.into(), atlas_region);
            Ok(atlas_region)
        }
    }

    pub fn clear_gui_images(&mut self) {
        self.gui_images.clear();
        self.gui_atlas.clear();
    }

    pub fn gui_pipeline(&self) -> &wgpu::RenderPipeline {
        self.gui_pipeline.as_ref().unwrap()
    }

    pub fn block_texture(&self) -> &DynamicTexture2D {
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
                    let atlas_region = Rectangle::from_span(atlas_offset, loaded_image.size());

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

        self.block_texture.rewrite(self.block_atlas.image());

        Ok(())
    }

    pub fn get_block_appearance(&self, block_type: &'static BlockType) -> &BlockAppearance {
        &self.block_appearances[&(block_type as *const _)]
    }

    pub fn get_block_image(&self, block: &Block, chunk_location: ChunkLocation, x: usize, y: usize) -> Option<&BlockImage> {
        self.get_block_appearance(block.block_type()).get_image(block, chunk_location, x, y)
    }

    pub fn block_pipeline(&self) -> &wgpu::RenderPipeline {
        self.block_pipeline.as_ref().unwrap()
    }

    pub fn item_texture(&self) -> &DynamicTexture2D {
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
            let atlas_region = Rectangle::from_span(atlas_offset, loaded_image.size());

            self.item_images.insert(item_type, atlas_region);
        }

        self.item_texture.rewrite(self.item_atlas.image());

        Ok(())
    }

    pub fn get_item_image(&self, item_type: &'static ItemType) -> Option<Rectangle<u32>> {
        self.item_images.get(&(item_type as *const _)).copied()
    }

    pub fn entity_texture(&self) -> &DynamicTexture2D {
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
            let atlas_region = Rectangle::from_span(atlas_offset, loaded_image.size());
            self.entity_texture.rewrite(self.entity_atlas.image());

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

    pub fn get_text<'a>(&'a self, key: &'a str) -> &'a str {
        match self.text_strings.get(key) {
            Some(string) => string,
            None => key,
        }
    }

    pub fn get_template_text(&self, key: &str, elements: &[&str]) -> String {
        let template_string = self.get_text(key).to_owned();
        elements
            .iter()
            .enumerate()
            .fold(template_string, |string, (index, &element)| {
                string.replace(&format!("{{{index}}}"), element)
            })
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

    pub fn reload_pipelines(&mut self, gfx: &Gfx) -> Result<(), String> {
        let texture_layout = DynamicTexture2D::create_layout(gfx.device());
        let camera_layout = Camera::create_layout(gfx.device());
        let gui_params_layout = GuiParams::create_layout(gfx.device());

        self.default_pipeline = Some(self.create_render_pipeline(
            gfx,
            "default",
            "Default Render Pipeline",
            &[
                Some(&texture_layout),
                Some(&camera_layout),
            ],
            Vertex2D::buffer_layout(),
        )?);
        self.gui_pipeline = Some(self.create_render_pipeline(
            gfx,
            "gui",
            "GUI Render Pipeline",
            &[
                Some(&texture_layout),
                None,
                Some(&gui_params_layout),
            ],
            GuiVertex::buffer_layout(),
        )?);
        self.block_pipeline = Some(self.create_render_pipeline(
            gfx,
            "block",
            "Chunk Render Pipeline",
            &[
                Some(&texture_layout),
                Some(&camera_layout),
            ],
            Vertex2D::buffer_layout(),
        )?);

        Ok(())
    }

    fn create_render_pipeline(
        &self,
        gfx: &Gfx,
        shader_name: &str,
        label: &str,
        bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
        vertex_layout: wgpu::VertexBufferLayout,
    ) -> Result<wgpu::RenderPipeline, String> {
        let shader_path = format!("shaders/{shader_name}.wgsl");
        let shader = gfx.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&shader_path),
            source: wgpu::ShaderSource::Wgsl(self.load_text(&shader_path)?.into()),
        });

        let pipeline_layout = gfx.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts,
            ..Default::default()
        });

        let render_pipeline = gfx.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gfx.surface().get_configuration().map_or(wgpu::TextureFormat::Rgba8UnormSrgb, |config| config.format),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(render_pipeline)
    }
}
