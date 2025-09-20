use innovus::gfx::Geometry;
use innovus::tools::Vector;
use crate::gui::GuiVertex;
use crate::tools::asset::AssetPool;

pub struct StringRenderer {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    placement: Vector<f32, 2>,
    text_color: Vector<f32, 4>,
    background_color: Vector<f32, 4>,
    string: String,
    geometry: Geometry<GuiVertex>,
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
            geometry: Geometry::new_render().unwrap(),
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

        const GLYPH_WIDTHS: [u32; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            4, 1, 3, 5, 5, 5, 5, 1, 3, 3, 5, 5, 1, 5, 1, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 1, 1, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 5, 5, 4, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 5, 3, 5, 5,
            3, 5, 5, 5, 5, 5, 4, 5, 5, 1, 3, 5, 2, 5, 5, 5,
            5, 5, 5, 5, 4, 5, 5, 5, 5, 5, 5, 3, 1, 3, 5, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 1, 5, 5, 5, 5, 1, 5, 5, 5, 3, 5, 5, 0, 5, 5,
            4, 3, 3, 3, 2, 5, 5, 3, 5, 3, 3, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        const OFFSETS: [(Vector<f32, 2>, Vector<u32, 2>); 4] = [
            (Vector([0.0, 0.0]), Vector([0, 8])), // Bottom left
            (Vector([0.0, 1.0]), Vector([0, 0])), // Top left
            (Vector([1.0, 1.0]), Vector([8, 0])), // Top right
            (Vector([1.0, 0.0]), Vector([8, 8])), // Bottom right
        ];

        if self.geometry.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();

            let glyph_max_size = 8.0;

            let text_width = 3.0 + self.string
                .bytes()
                .map(|character| (GLYPH_WIDTHS[character as usize] + 1) as f32)
                .sum::<f32>();
            let bounds_size = Vector([text_width, glyph_max_size]);
            let adjusted_offset = self.offset - self.placement * bounds_size;

            // Background rectangle
            faces.push([0, 1, 2]);
            faces.push([2, 3, 0]);
            for (vertex_offset, _) in OFFSETS {
                let total_offset = adjusted_offset + Vector([
                    vertex_offset.x() * text_width,
                    vertex_offset.y() * (glyph_max_size + 4.0),
                ]);
                vertices.push(GuiVertex::new(
                    self.anchor,
                    total_offset,
                    Some(self.background_color),
                    None,
                ));
            }

            // Foreground text
            let mut current_position = adjusted_offset + Vector([2.0, 2.0]);
            for character in self.string.bytes() {
                let index = vertices.len() as u32;
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
                let char_index = character as u32;
                let atlas_origin = Vector([char_index % 16, char_index / 16]) * 8;
                for (vertex_offset, atlas_offset) in OFFSETS {
                    vertices.push(GuiVertex::new(
                        self.anchor,
                        current_position + vertex_offset * glyph_max_size,
                        Some(self.text_color),
                        Some((atlas_origin + atlas_offset).map(|x| x as f32))
                    ));
                }
                let glyph_width = (GLYPH_WIDTHS[character as usize] + 1) as f32;
                current_position.set_x(current_position.x() + glyph_width);
            }

            self.geometry.add(&vertices, &faces);
        }

        assets.font_texture().bind();
        self.geometry.render();
    }
}
