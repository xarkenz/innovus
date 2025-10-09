use innovus::gfx::{Mesh, MeshRenderer};
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::tools::asset::AssetPool;

pub struct TextLine {
    fixed_point: Vector<f32, 2>,
    text_color: Vector<f32, 4>,
    background_color: Vector<f32, 4>,
    text: String,
    mesh: Mesh<GuiVertex>,
}

impl TextLine {
    pub fn new(
        fixed_point: Vector<f32, 2>,
        text_color: Vector<f32, 4>,
        background_color: Vector<f32, 4>,
        text: String,
    ) -> Self {
        Self {
            fixed_point,
            text_color,
            background_color,
            text,
            mesh: Mesh::new(),
        }
    }

    pub fn fixed_point(&self) -> Vector<f32, 2> {
        self.fixed_point
    }

    pub fn set_fixed_point(&mut self, fixed_point: Vector<f32, 2>) {
        if fixed_point != self.fixed_point {
            self.invalidate();
        }
        self.fixed_point = fixed_point;
    }

    pub fn text_color(&self) -> Vector<f32, 4> {
        self.text_color
    }

    pub fn set_text_color(&mut self, color: Vector<f32, 4>) {
        if color != self.text_color {
            self.invalidate();
        }
        self.text_color = color;
    }

    pub fn background_color(&self) -> Vector<f32, 4> {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Vector<f32, 4>) {
        if color != self.background_color {
            self.invalidate();
        }
        self.background_color = color;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        if text != self.text {
            self.invalidate();
        }
        self.text = text;
    }

    pub fn invalidate(&mut self) {
        self.mesh.clear();
    }

    pub fn append_to_mesh(&mut self, mesh: &mut Mesh<GuiVertex>, offset: Vector<f32, 2>, assets: &mut AssetPool) {
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
            let glyph_max_size = 12.0;

            let text_width = self.text
                .chars()
                .map(|character| glyph_info(character).1 + 1.0)
                .sum::<f32>()
                - 1.0;
            let text_size = Vector([text_width, glyph_max_size]);
            let text_offset = -self.fixed_point * text_size;

            if self.background_color.w() > 0.0 {
                // Background rectangle
                let background_size = text_size + Vector([2.0, 0.0]);
                let background_offset = text_offset - Vector([1.0, 0.0]);
                self.mesh.add(
                    &OFFSETS.map(|(vertex_offset, _)| {
                        GuiVertex::new(
                            background_offset + vertex_offset * background_size,
                            Some(self.background_color),
                            None,
                        )
                    }),
                    &[[0, 1, 2], [2, 3, 0]],
                );
            }

            let atlas_region = assets.get_gui_image("font/unicode_0").unwrap();
            let image_size = atlas_region.size().div(16);

            // Foreground text
            let mut current_offset = text_offset;
            for character in self.text.chars() {
                let (image_index, glyph_width) = glyph_info(character);
                let image_origin = atlas_region.min()
                    + Vector([image_index % 16, image_index / 16]) * image_size;
                self.mesh.add(
                    &OFFSETS.map(|(vertex_offset, atlas_offset)| {
                        GuiVertex::new(
                            current_offset + vertex_offset.mul(glyph_max_size),
                            Some(self.text_color),
                            Some((image_origin + atlas_offset).map(|x| x as f32)),
                        )
                    }),
                    &[[0, 1, 2], [2, 3, 0]],
                );

                current_offset.set_x(current_offset.x() + glyph_width + 1.0);
            }
        }

        let slice = mesh.add_mesh(&self.mesh);
        for vertex in mesh.slice_vertices_mut(slice) {
            vertex.offset += offset;
        }
    }
}

pub struct TextLineRenderer {
    text_line: TextLine,
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    mesh: MeshRenderer<GuiVertex>,
}

impl TextLineRenderer {
    pub fn create(text_line: TextLine, anchor: Vector<f32, 2>, offset: Vector<f32, 2>) -> Self {
        Self {
            text_line,
            anchor,
            offset,
            mesh: MeshRenderer::create().unwrap(),
        }
    }

    pub fn data(&self) -> &TextLine {
        &self.text_line
    }

    pub fn data_mut(&mut self) -> &mut TextLine {
        &mut self.text_line
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor;
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector<f32, 2>) {
        self.offset = offset;
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        if !self.text_line.text().is_empty() {
            self.mesh.clear();
            self.text_line.append_to_mesh(
                self.mesh.data_mut(),
                self.offset,
                assets,
            );
            self.mesh.upload_buffers();

            assets.gui_texture().bind();
            assets.gui_shaders().set_uniform("anchor", self.anchor);
            self.mesh.render();
        }
    }
}
