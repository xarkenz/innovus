use crate::world::block::{Block, BlockSide, BlockType};

pub mod types;

pub use types::ITEM_TYPES;

pub struct ItemType {
    name: &'static str,
    max_count: u32,
    block_type: Option<&'static BlockType>,
    right_click: fn(&Block, &Item, BlockSide) -> (Option<Block>, Option<Item>),
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn max_count(&self) -> u32 {
        self.max_count
    }

    pub fn block_type(&self) -> Option<&'static BlockType> {
        self.block_type
    }
}

impl PartialEq for ItemType {
    fn eq(&self, other: &Self) -> bool {
        // Comparing pointers is sufficient; only the static ItemType objects should be used.
        self as *const Self == other as *const Self
    }
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemType({})", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    item_type: &'static ItemType,
    count: u32,
}

impl Item {
    pub fn new(item_type: &'static ItemType, count: u32) -> Self {
        Self {
            item_type,
            count,
        }
    }

    pub fn item_type(&self) -> &'static ItemType {
        self.item_type
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn decrement_count(&self) -> Self {
        let mut item = self.clone();
        item.count = item.count.saturating_sub(1);
        if item.count == 0 {
            Self::new(&types::AIR, 0)
        }
        else {
            item
        }
    }

    pub fn handle_right_click(&self, target_block: &Block, side: BlockSide) -> (Option<Block>, Option<Self>) {
        (self.item_type.right_click)(target_block, self, side)
    }
}
