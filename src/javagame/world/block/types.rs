use crate::tools::*;

use super::*;

pub const BLOCK_RECT: Rectangle<i32> = Rectangle::new(Vector([0, 0]), Vector([32, 32]));

pub fn connect_never(_this: &BlockType, _that: &BlockType) -> bool {
    false
}

pub fn connect_full_block(_this: &BlockType, that: &BlockType) -> bool {
    that.is_full_block
}

pub fn connect_same(this: &BlockType, that: &BlockType) -> bool {
    that.name == this.name
}

pub static BLOCK_TYPES: &[&BlockType] = &[
    &AIR,
    &COPPER_BLOCK,
    &COPPER_WIRE,
    &DIRT,
    &GRASSY_DIRT,
    &PHYLUMUS_BLOCK,
    &SLATE,
    &STONE,
];

pub static AIR: BlockType = BlockType {
    name: "air",
    attributes: &[],
    colliders: &[],
    is_full_block: false,
    light_emission: 0,
    connector: connect_never,
};
pub static COPPER_BLOCK: BlockType = BlockType {
    name: "copper_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same,
};
pub static COPPER_WIRE: BlockType = BlockType {
    name: "copper_wire",
    attributes: &[],
    colliders: &[BLOCK_RECT], // FIXME
    is_full_block: false,
    light_emission: 0,
    connector: |this, that| that.name == this.name || that.name == COPPER_BLOCK.name,
};
pub static DIRT: BlockType = BlockType {
    name: "dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static GRASSY_DIRT: BlockType = BlockType {
    name: "grassy_dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static PHYLUMUS_BLOCK: BlockType = BlockType {
    name: "phylumus_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 15,
    connector: connect_same,
};
pub static SLATE: BlockType = BlockType {
    name: "slate",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static STONE: BlockType = BlockType {
    name: "stone",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
