use innovus::tools::*;
use crate::world::item::{Item, ItemType};

pub mod types;
mod chunk;
pub mod preview;

pub use types::BLOCK_TYPES;
pub use chunk::*;

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub enum BlockSide {
    #[default]
    None = 0,
    Left = 1,
    Right = 2,
    Bottom = 3,
    Top = 4,
}

impl BlockSide {
    pub fn from_position(position: Vector<f32, 2>) -> Self {
        let x = position.x().rem_euclid(1.0);
        let y = position.y().rem_euclid(1.0);
        match (y > x, x + y > 1.0) {
            (false, false) => Self::Bottom,
            (false, true) => Self::Right,
            (true, false) => Self::Left,
            (true, true) => Self::Top,
        }
    }
}

#[derive(Debug)]
pub enum AttributeType {
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    String(&'static str),
    Enum {
        side_default_values: [u8; 5],
        value_names: &'static [&'static str],
    },
}

impl AttributeType {
    pub fn description(&self) -> String {
        match *self {
            Self::Bool(..) => "a boolean".into(),
            Self::U8(..) => "a small unsigned integer".into(),
            Self::I8(..) => "a small signed integer".into(),
            Self::U32(..) => "an unsigned integer".into(),
            Self::I32(..) => "a signed integer".into(),
            Self::String(..) => "a string".into(),
            Self::Enum { value_names, .. } => {
                format!("a string matching one of: '{}'", value_names.join("', '"))
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AttributeValue {
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    String(String),
}

impl AttributeValue {
    pub fn expect_bool(&self) -> bool {
        match self {
            &AttributeValue::Bool(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }

    pub fn expect_u8(&self) -> u8 {
        match self {
            &AttributeValue::U8(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }

    pub fn expect_i8(&self) -> i8 {
        match self {
            &AttributeValue::I8(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }

    pub fn expect_u32(&self) -> u32 {
        match self {
            &AttributeValue::U32(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }

    pub fn expect_i32(&self) -> i32 {
        match self {
            &AttributeValue::I32(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }

    pub fn expect_string(&self) -> &str {
        match self {
            AttributeValue::String(value) => value,
            _ => panic!("unexpected attribute type")
        }
    }
}

pub struct BlockType {
    name: &'static str,
    attributes: &'static [(&'static str, AttributeType)],
    item_type: Option<&'static ItemType>,
    colliders: &'static [Rectangle<i32>],
    palette_key: Option<&'static str>,
    is_full_block: fn(&Block) -> bool,
    light_emission: fn(&Block) -> u8,
    connects_to: fn(&Block, &Block) -> bool,
    right_click: fn(&Block, &Item, BlockSide) -> (Option<Block>, Option<Item>),
}

impl BlockType {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn attributes(&self) -> &'static [(&'static str, AttributeType)] {
        self.attributes
    }

    pub fn item_type(&self) -> Option<&'static ItemType> {
        self.item_type
    }

    pub fn colliders(&self) -> &'static [Rectangle<i32>] {
        self.colliders
    }

    pub fn palette_key(&self) -> Option<&'static str> {
        self.palette_key
    }

    pub fn get_attribute_info(&self, name: &str) -> Option<(usize, &AttributeType)> {
        self.attributes
            .iter()
            .enumerate()
            .find_map(|(index, &(attribute_name, ref attribute_type))| {
                (name == attribute_name).then_some((index, attribute_type))
            })
    }

    pub fn default_attributes(&self, side: BlockSide) -> Box<[AttributeValue]> {
        self.attributes
            .iter()
            .map(|(_, attribute_type)| match attribute_type {
                &AttributeType::Bool(b) => AttributeValue::Bool(b),
                &AttributeType::U8(n) => AttributeValue::U8(n),
                &AttributeType::I8(n) => AttributeValue::I8(n),
                &AttributeType::U32(n) => AttributeValue::U32(n),
                &AttributeType::I32(n) => AttributeValue::I32(n),
                &AttributeType::String(s) => AttributeValue::String(s.into()),
                &AttributeType::Enum { side_default_values: default_values, .. } => {
                    AttributeValue::U8(default_values[side as usize])
                }
            })
            .collect()
    }
}

impl PartialEq for BlockType {
    fn eq(&self, other: &Self) -> bool {
        // Comparing pointers is sufficient; only the static BlockType objects should be used.
        self as *const Self == other as *const Self
    }
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockType({})", self.name)
    }
}

pub const QUADRANT_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.5, 0.0]), // Bottom right
];
pub const QUADRANT_VERTEX_OFFSETS: [Vector2f; 4] = [
    Vector([0.0, 0.0]), // Bottom left
    Vector([0.0, 0.5]), // Top left
    Vector([0.5, 0.5]), // Top right
    Vector([0.5, 0.0]), // Bottom right
];
pub const VERTICES_PER_BLOCK: usize = QUADRANT_OFFSETS.len() * QUADRANT_VERTEX_OFFSETS.len();

#[derive(Clone, Debug)]
pub struct Block {
    block_type: &'static BlockType,
    attributes: Box<[AttributeValue]>,
}

impl Block {
    pub fn new(block_type: &'static BlockType, side: BlockSide) -> Self {
        Self {
            block_type,
            attributes: block_type.default_attributes(side),
        }
    }

    pub fn block_type(&self) -> &'static BlockType {
        self.block_type
    }

    pub fn attributes(&self) -> &[AttributeValue] {
        &self.attributes
    }

    pub fn attribute_value(&self, index: usize) -> &AttributeValue {
        &self.attributes[index]
    }

    pub fn set_attribute_value(&mut self, index: usize, value: AttributeValue) {
        self.attributes[index] = value;
    }

    pub fn is_full_block(&self) -> bool {
        (self.block_type.is_full_block)(self)
    }

    pub fn light_emission(&self) -> u8 {
        (self.block_type.light_emission)(self)
    }

    pub fn connects_to(&self, other: &Self) -> bool {
        (self.block_type.connects_to)(self, other)
    }

    pub fn handle_right_click(&self, held_item: &Item, side: BlockSide) -> (Option<Self>, Option<Item>) {
        (self.block_type.right_click)(self, held_item, side)
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(&types::AIR, Default::default())
    }
}
