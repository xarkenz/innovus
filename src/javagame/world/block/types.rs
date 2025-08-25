use crate::tools::*;

use super::*;

pub const BLOCK_RECT: Rectangle<i32> = Rectangle::new(Vector([0, 0]), Vector([32, 32]));

pub fn connect_never(this: &Block, that: &Block) -> bool {
    let _ = (this, that);
    false
}

pub fn connect_full_block(this: &Block, that: &Block) -> bool {
    let _ = this;
    that.block_type.is_full_block
}

pub fn connect_same_type(this: &Block, that: &Block) -> bool {
    that.block_type == this.block_type
}

pub fn connect_electricity(this: &Block, that: &Block) -> bool {
    let _ = this;
    that.block_type == &COPPER_WIRE || that.block_type == &COPPER_BLOCK ||
        that.block_type == &GOLD_WIRE || that.block_type == &GOLD_BLOCK
}

pub const BLOCK_TYPES: &[&BlockType] = &[
    &AIR,
    &ALUMINUM_BLOCK,
    &AMETHYST_BLOCK,
    &AMPLIFITE_BLOCK,
    &COAL_BLOCK,
    &COBALT_BLOCK,
    &COBBLES,
    &COPPER_BLOCK,
    &COPPER_WIRE,
    &CORRUPTITE_BLOCK,
    &DIAMOND_BLOCK,
    &DIRT,
    &EMERALD_BLOCK,
    &FLAMARITE_BLOCK,
    &FRIGIDITE_BLOCK,
    &GLASS,
    &GOLD_BLOCK,
    &GOLD_WIRE,
    &GRASSY_DIRT,
    &HONEY_CRYSTAL_BLOCK,
    &IRON_BLOCK,
    &LUMINITE_BLOCK,
    &MAGMIUM_BLOCK,
    &OBSIDIAN_BLOCK,
    &PHYLUMUS_BLOCK,
    &PIPE,
    &PLATINUM_BLOCK,
    &QUARTZ_BLOCK,
    &SAND,
    &SANDSTONE,
    &SLATE,
    &STEEL_BLOCK,
    &STONE,
    &TITANIUM_BLOCK,
    &TURQUOISE_BLOCK,
    &VERSATILIUM_BLOCK,
    &VOLTAGITE_BLOCK,
];

pub static AIR: BlockType = BlockType {
    name: "air",
    attributes: &[],
    colliders: &[],
    is_full_block: false,
    light_emission: 0,
    connector: connect_never,
};
pub static ALUMINUM_BLOCK: BlockType = BlockType {
    name: "aluminum_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static AMETHYST_BLOCK: BlockType = BlockType {
    name: "amethyst_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static AMPLIFITE_BLOCK: BlockType = BlockType {
    name: "amplifite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static COAL_BLOCK: BlockType = BlockType {
    name: "coal_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static COBALT_BLOCK: BlockType = BlockType {
    name: "cobalt_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static COBBLES: BlockType = BlockType {
    name: "cobbles",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static COPPER_BLOCK: BlockType = BlockType {
    name: "copper_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static COPPER_WIRE: BlockType = BlockType {
    name: "copper_wire",
    attributes: &[],
    colliders: &[],
    is_full_block: false,
    light_emission: 0,
    connector: connect_electricity,
};
pub static CORRUPTITE_BLOCK: BlockType = BlockType {
    name: "corruptite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static DIAMOND_BLOCK: BlockType = BlockType {
    name: "diamond_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static DIRT: BlockType = BlockType {
    name: "dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static EMERALD_BLOCK: BlockType = BlockType {
    name: "emerald_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static FLAMARITE_BLOCK: BlockType = BlockType {
    name: "flamarite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static FRIGIDITE_BLOCK: BlockType = BlockType {
    name: "frigidite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static GLASS: BlockType = BlockType {
    name: "glass",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static GOLD_BLOCK: BlockType = BlockType {
    name: "gold_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static GOLD_WIRE: BlockType = BlockType {
    name: "gold_wire",
    attributes: &[],
    colliders: &[],
    is_full_block: false,
    light_emission: 0,
    connector: connect_electricity,
};
pub static GRASSY_DIRT: BlockType = BlockType {
    name: "grassy_dirt",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static HONEY_CRYSTAL_BLOCK: BlockType = BlockType {
    name: "honey_crystal_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static IRON_BLOCK: BlockType = BlockType {
    name: "iron_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static LUMINITE_BLOCK: BlockType = BlockType {
    name: "luminite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static MAGMIUM_BLOCK: BlockType = BlockType {
    name: "magmium_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static OBSIDIAN_BLOCK: BlockType = BlockType {
    name: "obsidian_block",
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
    connector: connect_full_block,
};
pub static PIPE: BlockType = BlockType {
    name: "pipe",
    attributes: &[],
    colliders: &[],
    is_full_block: false,
    light_emission: 0,
    connector: connect_same_type,
};
pub static PLATINUM_BLOCK: BlockType = BlockType {
    name: "platinum_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static QUARTZ_BLOCK: BlockType = BlockType {
    name: "quartz_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static SAND: BlockType = BlockType {
    name: "sand",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static SANDSTONE: BlockType = BlockType {
    name: "sandstone",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static SLATE: BlockType = BlockType {
    name: "slate",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static STEEL_BLOCK: BlockType = BlockType {
    name: "steel_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static STONE: BlockType = BlockType {
    name: "stone",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_full_block,
};
pub static TITANIUM_BLOCK: BlockType = BlockType {
    name: "titanium_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static TURQUOISE_BLOCK: BlockType = BlockType {
    name: "turquoise_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static VERSATILIUM_BLOCK: BlockType = BlockType {
    name: "versatilium_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
pub static VOLTAGITE_BLOCK: BlockType = BlockType {
    name: "voltagite_block",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: true,
    light_emission: 0,
    connector: connect_same_type,
};
