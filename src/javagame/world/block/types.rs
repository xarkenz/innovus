use crate::tools::*;

use super::*;

pub const BLOCK_RECT: Rectangle<i32> = Rectangle::new(0, 0, 32, 32);

pub fn connect_never(_this: &BlockType, _that: &BlockType) -> bool {
    false
}

pub fn connect_full_block(_this: &BlockType, that: &BlockType) -> bool {
    that.full_block
}

pub fn connect_same(this: &BlockType, that: &BlockType) -> bool {
    that.name == this.name
}

pub static AIR: BlockType = BlockType {
    name: "air",
    attributes: &[],
    colliders: &[],
    full_block: false,
    light_emission: 0,
    connector: connect_never,
};
pub static COPPER_BLOCK: BlockType = BlockType {
    name: "copper_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 0,
    connector: connect_same,
};
pub static COPPER_WIRE: BlockType = BlockType {
    name: "copper_wire",
    attributes: &[],
    colliders: &[BLOCK_RECT], // FIXME
    full_block: false,
    light_emission: 0,
    connector: |this, that| that.name == this.name || that.name == "copper_block",
};
pub static DIRT: BlockType = BlockType {
    name: "dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static GRASSY_DIRT: BlockType = BlockType {
    name: "grassy_dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static PHYLUMUS_BLOCK: BlockType = BlockType {
    name: "phylumus_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 15,
    connector: connect_same,
};
pub static SLATE: BlockType = BlockType {
    name: "slate",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static STONE: BlockType = BlockType {
    name: "stone",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
