use std::hash::{Hash, Hasher};
use json::JsonValue;
use innovus::tools::{Rectangle, Vector};
use crate::tools::noise::SimpleHasher;
use crate::world::block::{AttributeType, AttributeValue, Block, BlockType, Chunk, ChunkLocation, ChunkMap};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SimpleShape {
    OutwardCorners = 0,
    Horizontal = 1,
    Vertical = 2,
    InwardCorners = 3,
    Fill = 4,
}

impl std::fmt::Display for SimpleShape {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OutwardCorners => f.write_str("outward_corners"),
            Self::Horizontal => f.write_str("horizontal"),
            Self::Vertical => f.write_str("vertical"),
            Self::InwardCorners => f.write_str("inward_corners"),
            Self::Fill => f.write_str("fill"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BlockImageFormat {
    Single,
    ConnectedSimple {
        shape_offsets: [u32; 5],
    },
}

impl BlockImageFormat {
    pub fn shape_count(&self) -> u32 {
        match self {
            Self::Single => 1,
            Self::ConnectedSimple { shape_offsets } => shape_offsets.len() as u32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockImage {
    key: String,
    atlas_offset: Vector<u32, 2>,
    size: u32,
    format: BlockImageFormat,
}

impl BlockImage {
    pub fn parse(key: String, metadata: &JsonValue, defaults: Option<&Self>) -> Result<Self, String> {
        let size;
        if metadata["size"].is_null() {
            let Some(defaults) = defaults else {
                return Err(format!("missing size property for {key}"));
            };
            size = defaults.size;
        }
        else if let Some(size_value) = metadata["size"].as_u32() {
            if size_value == 0 {
                return Err(format!("invalid size property for {key}: must be a positive integer"));
            }
            size = size_value;
        }
        else {
            return Err(format!("invalid size property for {key}: must be a positive integer"));
        }

        let format;
        if metadata["format"].is_null() {
            let Some(defaults) = defaults else {
                return Err(format!("missing format property for {key}"));
            };
            format = defaults.format.clone();
        }
        else if let Some(format_name) = metadata["format"].as_str() {
            match format_name {
                "single" => {
                    format = BlockImageFormat::Single;
                }
                "connected_simple" => {
                    let mut shape_offsets = [5; 5];
                    for (offset, shape) in metadata["shapes"].members().enumerate() {
                        let shape = match shape.as_str() {
                            Some("outward_corners") => SimpleShape::OutwardCorners,
                            Some("horizontal") => SimpleShape::Horizontal,
                            Some("vertical") => SimpleShape::Vertical,
                            Some("inward_corners") => SimpleShape::InwardCorners,
                            Some("fill") => SimpleShape::Fill,
                            Some(shape_name) => {
                                return Err(format!("invalid shapes property for {key}: unrecognized shape '{shape_name}'"));
                            }
                            None => {
                                return Err(format!("invalid shapes property for {key}: all shapes must be strings"));
                            }
                        };
                        if shape_offsets[shape as usize] < 5 {
                            return Err(format!("invalid shapes property for {key}: duplicate shape '{shape}'"));
                        }
                        shape_offsets[shape as usize] = offset as u32;
                    }
                    if shape_offsets.iter().any(|&shape_offset| shape_offset >= 5) {
                        return Err(format!("invalid shapes property for {key}: one or more shapes missing"));
                    }

                    format = BlockImageFormat::ConnectedSimple {
                        shape_offsets,
                    };
                }
                _ => {
                    return Err(format!("invalid format property for {key}: unrecognized format '{format_name}'"));
                }
            }
        }
        else {
            return Err(format!("invalid format property for {key}: must be a string"));
        }

        Ok(Self {
            key,
            atlas_offset: Vector::zero(),
            size,
            format,
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_atlas_region(&mut self, atlas_region: Rectangle<u32>) -> Result<(), String> {
        let expected_width = self.size * self.format.shape_count();
        let expected_height = self.size;
        if atlas_region.width() != expected_width || atlas_region.height() != expected_height {
            return Err(format!("unexpected image size for {}: expected {expected_width}x{expected_height}, got {}x{}", self.key, atlas_region.width(), atlas_region.height()));
        }

        self.atlas_offset = atlas_region.min();
        Ok(())
    }

    pub fn get_quadrant_atlas_offsets(&self, chunk_map: &ChunkMap, chunk: &Chunk, x: usize, y: usize) -> [Vector<u32, 2>; 4] {
        if let BlockImageFormat::Single = self.format {
            return [self.atlas_offset; 4];
        }

        let block = chunk.block_at(x, y);

        let left_x = x as isize - 1;
        let right_x = x as isize + 1;
        let down_y = y as isize - 1;
        let up_y = y as isize + 1;

        let left_connect = self.get_connect(chunk_map, chunk, block, left_x, y as isize);
        let right_connect = self.get_connect(chunk_map, chunk, block, right_x, y as isize);
        let down_connect = self.get_connect(chunk_map, chunk, block, x as isize, down_y);
        let up_connect = self.get_connect(chunk_map, chunk, block, x as isize, up_y);

        let shape_offsets: [u32; 4] = [
            self.get_shape_offset(chunk_map, chunk, block, left_x, up_y, left_connect, up_connect),
            self.get_shape_offset(chunk_map, chunk, block, right_x, up_y, right_connect, up_connect),
            self.get_shape_offset(chunk_map, chunk, block, left_x, down_y, left_connect, down_connect),
            self.get_shape_offset(chunk_map, chunk, block, right_x, down_y, right_connect, down_connect),
        ];

        shape_offsets.map(|shape_offset| Vector([
            self.atlas_offset.x() + shape_offset * self.size,
            self.atlas_offset.y(),
        ]))
    }

    fn get_shape_offset(&self, chunk_map: &ChunkMap, chunk: &Chunk, this_block: &Block, that_x: isize, that_y: isize, x_connect: bool, y_connect: bool) -> u32 {
        match &self.format {
            BlockImageFormat::Single => {
                0
            }
            BlockImageFormat::ConnectedSimple { shape_offsets } => {
                let shape = match (x_connect, y_connect) {
                    (false, false) => SimpleShape::OutwardCorners,
                    (true, false) => SimpleShape::Horizontal,
                    (false, true) => SimpleShape::Vertical,
                    (true, true) => {
                        if self.get_connect(chunk_map, chunk, this_block, that_x, that_y) {
                            SimpleShape::Fill
                        }
                        else {
                            SimpleShape::InwardCorners
                        }
                    }
                };

                shape_offsets[shape as usize]
            }
        }
    }

    fn get_connect(&self, chunk_map: &ChunkMap, chunk: &Chunk, this_block: &Block, that_x: isize, that_y: isize) -> bool {
        chunk
            .with_block_slot(that_x, that_y, chunk_map, |that_slot| {
                this_block.connects_to(that_slot.block())
            })
            .unwrap_or(true)
    }
}

fn parse_attribute_value(attribute_type: &AttributeType, value: &JsonValue) -> Option<AttributeValue> {
    match *attribute_type {
        AttributeType::Bool(..) => value.as_bool().map(AttributeValue::Bool),
        AttributeType::U8(..) => value.as_u8().map(AttributeValue::U8),
        AttributeType::I8(..) => value.as_i8().map(AttributeValue::I8),
        AttributeType::U32(..) => value.as_u32().map(AttributeValue::U32),
        AttributeType::I32(..) => value.as_i32().map(AttributeValue::I32),
        AttributeType::String(..) => value.as_str().map(str::to_owned).map(AttributeValue::String),
        AttributeType::Enum { value_names, .. } => {
            value.as_str().and_then(|string| {
                value_names
                    .iter()
                    .position(|&value_name| string == value_name)
                    .map(|index| AttributeValue::U8(index as u8))
            })
        }
    }
}

#[derive(Debug)]
struct Condition {
    attributes: Vec<(usize, AttributeValue)>,
}

impl Condition {
    fn parse(data: &JsonValue, block_type: &BlockType) -> Result<Self, String> {
        let mut attributes = Vec::new();

        // Note that data being null results in this loop being skipped.
        for (name, value) in data.entries() {
            let Some((index, attribute_type)) = block_type.get_attribute_info(name) else {
                return Err(format!("block type '{}' has no attribute named '{name}'", block_type.name));
            };

            let Some(value) = parse_attribute_value(attribute_type, value) else {
                return Err(format!("attribute '{name}' for block type '{}' expects {}", block_type.name, attribute_type.description()));
            };

            attributes.push((index, value));
        }

        Ok(Self {
            attributes,
        })
    }

    fn accepts(&self, block: &Block) -> bool {
        self.attributes
            .iter()
            .all(|&(index, ref accepted_value)| {
                block.attribute_value(index) == accepted_value
            })
    }
}

#[derive(Debug)]
struct ImageSelector {
    images: Vec<BlockImage>,
}

impl ImageSelector {
    fn parse<F>(list: &JsonValue, block_type: &BlockType, mut get_image: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Result<BlockImage, String>,
    {
        let mut images = Vec::new();

        for image_key in list.members() {
            let Some(image_key) = image_key.as_str() else {
                return Err(format!("all image keys for block type '{}' must be strings", block_type.name));
            };
            let image_key = image_key.replace("{block_type}", block_type.name);

            images.push(get_image(&image_key)?);
        }

        if images.is_empty() {
            return Err(format!("expected a list of one or more image keys for block type '{}'", block_type.name));
        }

        Ok(Self {
            images,
        })
    }

    fn select_image(&self, chunk_location: ChunkLocation, x: usize, y: usize) -> &BlockImage {
        if self.images.len() == 1 {
            &self.images[0]
        }
        else {
            let mut hasher = SimpleHasher::default();
            chunk_location.x().hash(&mut hasher);
            chunk_location.y().hash(&mut hasher);
            x.hash(&mut hasher);
            y.hash(&mut hasher);

            &self.images[hasher.finish() as usize % self.images.len()]
        }
    }
}

#[derive(Debug)]
pub struct BlockAppearance {
    states: Vec<(Condition, ImageSelector)>,
}

impl BlockAppearance {
    pub fn parse<F>(data: &JsonValue, block_type: &BlockType, mut get_image: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Result<BlockImage, String>,
    {
        let mut states = Vec::new();

        for state in data.members() {
            let condition = Condition::parse(&state["condition"], block_type)?;
            let image_selector = ImageSelector::parse(&state["images"], block_type, &mut get_image)?;

            states.push((condition, image_selector));
        }

        Ok(Self {
            states,
        })
    }

    pub fn get_image(&self, block: &Block, chunk_location: ChunkLocation, x: usize, y: usize) -> Option<&BlockImage> {
        self.states
            .iter()
            .find_map(|(condition, image_selector)| {
                condition.accepts(block).then(|| image_selector.select_image(chunk_location, x, y))
            })
    }
}
