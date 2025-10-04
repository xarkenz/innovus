use innovus::gfx::MeshRenderer;
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::tools::asset::AssetPool;

pub struct StringRenderer {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    placement: Vector<f32, 2>,
    text_color: Vector<f32, 4>,
    background_color: Vector<f32, 4>,
    string: String,
    geometry: MeshRenderer<GuiVertex>,
}

impl StringRenderer {
    pub fn new(
        anchor: Vector<f32, 2>,
        offset: Vector<f32, 2>,
        placement: Vector<f32, 2>,
        text_color: Vector<f32, 4>,
        background_color: Vector<f32, 4>,
        string: String,
    ) -> Self {
        Self {
            anchor,
            offset,
            placement,
            text_color,
            background_color,
            string,
            geometry: MeshRenderer::create().unwrap(),
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

    pub fn string(&self) -> &str {
        &self.string
    }

    pub fn set_string(&mut self, string: String) {
        self.string = string;
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.geometry.clear();
    }

    pub fn render(&mut self, assets: &AssetPool) {
        if self.string.is_empty() {
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

        if self.geometry.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();

            let glyph_max_size = 12.0;

            let text_width = self.string
                .chars()
                .map(|character| glyph_info(character).1 + 1.0)
                .sum::<f32>()
                - 1.0;
            let text_size = Vector([text_width, glyph_max_size]);
            let text_offset = self.offset - self.placement * text_size;
            let background_size = text_size + Vector([2.0, 0.0]);
            let background_offset = text_offset - Vector([1.0, 0.0]);

            // Background rectangle
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

            // Foreground text
            let mut current_offset = text_offset;
            for character in self.string.chars() {
                let index = vertices.len() as u32;
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
                let (glyph_index, glyph_width) = glyph_info(character);
                let atlas_origin = Vector([glyph_index % 16, glyph_index / 16]) * 12;
                for (vertex_offset, atlas_offset) in OFFSETS {
                    vertices.push(GuiVertex::new(
                        self.anchor,
                        current_offset + vertex_offset * glyph_max_size,
                        Some(self.text_color),
                        Some((atlas_origin + atlas_offset).map(|x| x as f32))
                    ));
                }
                current_offset.set_x(current_offset.x() + glyph_width + 1.0);
            }

            self.geometry.add(&vertices, &faces);
        }

        assets.font_texture().bind();
        self.geometry.render();
    }
}
