use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::Vector;
use crate::tools::asset::AssetPool;

pub struct StringRenderer {
    position: Vector<f32, 2>,
    text_color: Vector<f32, 4>,
    background_color: Vector<f32, 4>,
    string: String,
    geometry: Geometry<Vertex2D>,
}

impl StringRenderer {
    pub fn new(position: Vector<f32, 2>, text_color: Vector<f32, 4>, background_color: Vector<f32, 4>, string: String) -> Self {
        Self {
            position,
            text_color,
            background_color,
            string,
            geometry: Geometry::new_render().unwrap(),
        }
    }

    pub fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector<f32, 2>) {
        self.position = position;
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
        const GLYPH_WIDTHS: [u32; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 1, 3, 5, 5, 5, 5, 1, 3, 3, 5, 5, 1, 5, 1, 5,
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

            let pixel_size = 1.0 / 16.0;
            let glyph_max_size = 8.0 * pixel_size;

            faces.push([0, 1, 2]);
            faces.push([2, 3, 0]);
            let text_width = 3.0 * pixel_size + self.string
                .bytes()
                .map(|character| (GLYPH_WIDTHS[character as usize] + 1) as f32 * pixel_size)
                .sum::<f32>();
            for (world_offset, _) in OFFSETS {
                let position = self.position + Vector([
                    world_offset.x() * text_width,
                    world_offset.y() * (glyph_max_size + 4.0 * pixel_size),
                ]);
                vertices.push(Vertex2D::new(
                    [position.x(), position.y(), -1.0],
                    Some(self.background_color.0),
                    None,
                ));
            }

            let mut world_position = self.position + Vector([2.0 * pixel_size; 2]);
            for character in self.string.bytes() {
                let index = vertices.len() as u32;
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
                let char_index = character as u32;
                let atlas_origin = Vector([char_index % 16, char_index / 16]) * 8;
                for (world_offset, atlas_offset) in OFFSETS {
                    let position = world_position + world_offset * glyph_max_size;
                    vertices.push(Vertex2D::new(
                        [position.x(), position.y(), 0.0],
                        Some(self.text_color.0),
                        Some((atlas_origin + atlas_offset).map(|x| x as f32).0)
                    ));
                }
                let glyph_width = (GLYPH_WIDTHS[character as usize] + 1) as f32 * pixel_size;
                world_position.set_x(world_position.x() + glyph_width);
            }

            self.geometry.add(&vertices, &faces);
        }

        assets.font_texture().bind();
        self.geometry.render();
    }
}
