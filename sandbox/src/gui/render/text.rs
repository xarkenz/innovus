use innovus::gfx::MeshRenderer;
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::tools::asset::AssetPool;

pub struct TextLineRenderer {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    placement: Vector<f32, 2>,
    text_color: Vector<f32, 4>,
    background_color: Vector<f32, 4>,
    text: String,
    mesh: MeshRenderer<GuiVertex>,
}

impl TextLineRenderer {
    pub fn new(
        anchor: Vector<f32, 2>,
        offset: Vector<f32, 2>,
        placement: Vector<f32, 2>,
        text_color: Vector<f32, 4>,
        background_color: Vector<f32, 4>,
        text: String,
    ) -> Self {
        Self {
            anchor,
            offset,
            placement,
            text_color,
            background_color,
            text,
            mesh: MeshRenderer::create().unwrap(),
        }
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor;
        self.invalidate();
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, position: Vector<f32, 2>) {
        self.offset = position;
        self.invalidate();
    }

    pub fn placement(&self) -> Vector<f32, 2> {
        self.placement
    }

    pub fn set_placement(&mut self, position: Vector<f32, 2>) {
        self.placement = position;
        self.invalidate();
    }

    pub fn text_color(&self) -> Vector<f32, 4> {
        self.text_color
    }

    pub fn set_text_color(&mut self, color: Vector<f32, 4>) {
        self.text_color = color;
        self.invalidate();
    }

    pub fn background_color(&self) -> Vector<f32, 4> {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Vector<f32, 4>) {
        self.background_color = color;
        self.invalidate();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, string: String) {
        self.text = string;
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.mesh.clear();
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        if self.text.is_empty() {
            return;
        }

        fn glyph_info(character: char) -> (u32, f32) {
            const INVALID_GLYPH_WIDTH: u32 = 6;
            const X: u32 = u32::MAX;
            const GLYPH_WIDTHS: [u32; 256] = [
                X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
                X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
                4, 1, 3, 5, 5, 5, 5, 1, 3, 3, 5, 5, 2, 5, 1, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 1, 2, 5, 5, 5, 5,
                6, 5, 5, 5, 5, 5, 5, 5, 5, 3, 5, 5, 4, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 5, 3, 5, 5,
                3, 5, 5, 5, 5, 5, 4, 5, 5, 1, 2, 5, 2, 5, 5, 5,
                5, 5, 4, 5, 4, 5, 5, 5, 5, 5, 5, 3, 1, 3, 5, X,
                X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
                X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
                4, 1, 5, 5, 5, 5, 1, 5, 3, 7, 4, 5, 5, 0, 7, 5,
                4, 5, 4, 4, 2, 6, 6, 1, 2, 3, 4, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 3, 3, 3, 3,
                6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 3, 3, 3, 3,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ];
            match GLYPH_WIDTHS.get(character as usize) {
                Some(&width) if width != X => (character as u32, width as f32),
                _ => (0, INVALID_GLYPH_WIDTH as f32)
            }
        }
        const OFFSETS: [(Vector<f32, 2>, Vector<u32, 2>); 4] = [
            (Vector([0.0, 0.0]), Vector([0, 12])), // Bottom left
            (Vector([0.0, 1.0]), Vector([0, 0])), // Top left
            (Vector([1.0, 1.0]), Vector([12, 0])), // Top right
            (Vector([1.0, 0.0]), Vector([12, 12])), // Bottom right
        ];

        if self.mesh.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();

            let glyph_max_size = 12.0;

            let text_width = self.text
                .chars()
                .map(|character| glyph_info(character).1 + 1.0)
                .sum::<f32>()
                - 1.0;
            let text_size = Vector([text_width, glyph_max_size]);
            let text_offset = self.offset - self.placement * text_size;

            if self.background_color.w() > 0.0 {
                // Background rectangle
                let background_size = text_size + Vector([2.0, 0.0]);
                let background_offset = text_offset - Vector([1.0, 0.0]);
                faces.push([0, 1, 2]);
                faces.push([2, 3, 0]);
                for (vertex_offset, _) in OFFSETS {
                    let total_offset = background_offset + vertex_offset * background_size;
                    vertices.push(GuiVertex::new(
                        self.anchor,
                        total_offset,
                        Some(self.background_color),
                        None,
                    ));
                }
            }

            let atlas_region = assets.get_gui_image("font/unicode_0").unwrap();
            let image_size = atlas_region.size() / 16;

            // Foreground text
            let mut current_offset = text_offset;
            for character in self.text.chars() {
                let index = vertices.len() as u32;
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
                let (image_index, glyph_width) = glyph_info(character);
                let image_origin = atlas_region.min()
                    + Vector([image_index % 16, image_index / 16]) * image_size;
                for (vertex_offset, atlas_offset) in OFFSETS {
                    vertices.push(GuiVertex::new(
                        self.anchor,
                        current_offset + vertex_offset * glyph_max_size,
                        Some(self.text_color),
                        Some((image_origin + atlas_offset).map(|x| x as f32)),
                    ));
                }
                current_offset.set_x(current_offset.x() + glyph_width + 1.0);
            }

            self.mesh.add(&vertices, &faces);
        }

        assets.gui_texture().bind();
        self.mesh.render();
    }
}
