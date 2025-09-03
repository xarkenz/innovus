use crate::tools::*;

use super::*;

const BLOCK_RECT: Rectangle<i32> = Rectangle::new(Vector([0, 0]), Vector([32, 32]));

fn full_block_always(this: &Block) -> bool {
    let _ = this;
    true
}

fn full_block_never(this: &Block) -> bool {
    let _ = this;
    false
}

fn light_emission_0(this: &Block) -> u8 {
    let _ = this;
    0
}

fn light_emission_5(this: &Block) -> u8 {
    let _ = this;
    5
}

fn light_emission_15(this: &Block) -> u8 {
    let _ = this;
    15
}

fn connects_never(this: &Block, that: &Block) -> bool {
    let _ = (this, that);
    false
}

fn connects_to_full_block(this: &Block, that: &Block) -> bool {
    let _ = this;
    that.is_full_block()
}

fn connects_to_same_type(this: &Block, that: &Block) -> bool {
    that.block_type() == this.block_type()
}

fn connects_to_electricity(this: &Block, that: &Block) -> bool {
    let _ = this;
    that.block_type() == &COPPER_WIRE || that.block_type() == &COPPER_BLOCK ||
        that.block_type() == &GOLD_WIRE || that.block_type() == &GOLD_BLOCK ||
        that.block_type() == &VOLTAGITE_BATTERY
}

fn right_click_no_action(this: &Block, hand: &'static BlockType) -> Option<Block> {
    let _ = (this, hand);
    None
}

const DEFAULTS: BlockType = BlockType {
    name: "invalid",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    is_full_block: full_block_always,
    light_emission: light_emission_0,
    connects_to: connects_never,
    right_click: right_click_no_action,
};

pub const BLOCK_TYPES: &[&BlockType] = &[
    &AIR,
    &ALUMINUM_BLOCK,
    &AMETHYST_BLOCK,
    &AMETHYST_CRYSTAL,
    &AMETHYST_ORE,
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
    &IRON_BLOCK,
    &LUMINITE_BLOCK,
    &MAGMIUM_BLOCK,
    &OBSIDIAN_BLOCK,
    &PHYLUMUS_BLOCK,
    &PIPE,
    &QUARTZ_BLOCK,
    &QUARTZ_CRYSTAL,
    &QUARTZ_ORE,
    &SAND,
    &SANDSTONE,
    &SLATE,
    &STEEL_BLOCK,
    &STONE,
    &VERSATILIUM_BLOCK,
    &VOLTAGITE_BATTERY,
    &VOLTAGITE_BLOCK,
];

pub static AIR: BlockType = BlockType {
    name: "air",
    colliders: &[],
    is_full_block: full_block_never,
    right_click: |_, hand| {
        Some(Block::new(hand))
    },
    ..DEFAULTS
};
pub static ALUMINUM_BLOCK: BlockType = BlockType {
    name: "aluminum_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static AMETHYST_BLOCK: BlockType = BlockType {
    name: "amethyst_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static AMETHYST_CRYSTAL: BlockType = BlockType {
    name: "amethyst_crystal",
    attributes: &[
        ("wall", AttributeType::Enum { default_value: 0, value_names: &["bottom", "left", "right", "top"] }),
    ],
    colliders: &[],
    is_full_block: full_block_never,
    light_emission: light_emission_5,
    right_click: |block, _| {
        let mut block = block.clone();
        let wall = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((wall + 1) % 4));
        Some(block)
    },
    ..DEFAULTS
};
pub static AMETHYST_ORE: BlockType = BlockType {
    name: "amethyst_ore",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static AMPLIFITE_BLOCK: BlockType = BlockType {
    name: "amplifite_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COAL_BLOCK: BlockType = BlockType {
    name: "coal_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COBALT_BLOCK: BlockType = BlockType {
    name: "cobalt_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COBBLES: BlockType = BlockType {
    name: "cobbles",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static COPPER_BLOCK: BlockType = BlockType {
    name: "copper_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COPPER_WIRE: BlockType = BlockType {
    name: "copper_wire",
    colliders: &[],
    is_full_block: full_block_never,
    connects_to: connects_to_electricity,
    ..DEFAULTS
};
pub static CORRUPTITE_BLOCK: BlockType = BlockType {
    name: "corruptite_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static DIAMOND_BLOCK: BlockType = BlockType {
    name: "diamond_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static DIRT: BlockType = BlockType {
    name: "dirt",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static EMERALD_BLOCK: BlockType = BlockType {
    name: "emerald_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static FLAMARITE_BLOCK: BlockType = BlockType {
    name: "flamarite_block",
    light_emission: light_emission_5,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static FRIGIDITE_BLOCK: BlockType = BlockType {
    name: "frigidite_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GLASS: BlockType = BlockType {
    name: "glass",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GOLD_BLOCK: BlockType = BlockType {
    name: "gold_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GOLD_WIRE: BlockType = BlockType {
    name: "gold_wire",
    colliders: &[],
    is_full_block: full_block_never,
    connects_to: connects_to_electricity,
    ..DEFAULTS
};
pub static GRASSY_DIRT: BlockType = BlockType {
    name: "grassy_dirt",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static IRON_BLOCK: BlockType = BlockType {
    name: "iron_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static LUMINITE_BLOCK: BlockType = BlockType {
    name: "luminite_block",
    light_emission: light_emission_15,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static MAGMIUM_BLOCK: BlockType = BlockType {
    name: "magmium_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static OBSIDIAN_BLOCK: BlockType = BlockType {
    name: "obsidian_block",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static PHYLUMUS_BLOCK: BlockType = BlockType {
    name: "phylumus_block",
    light_emission: light_emission_15,
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static PIPE: BlockType = BlockType {
    name: "pipe",
    colliders: &[],
    is_full_block: full_block_never,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static QUARTZ_BLOCK: BlockType = BlockType {
    name: "quartz_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static QUARTZ_CRYSTAL: BlockType = BlockType {
    name: "quartz_crystal",
    attributes: &[
        ("wall", AttributeType::Enum { default_value: 0, value_names: &["bottom", "left", "right", "top"] }),
    ],
    colliders: &[],
    is_full_block: full_block_never,
    light_emission: light_emission_5,
    right_click: |block, _| {
        let mut block = block.clone();
        let wall = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((wall + 1) % 4));
        Some(block)
    },
    ..DEFAULTS
};
pub static QUARTZ_ORE: BlockType = BlockType {
    name: "quartz_ore",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SAND: BlockType = BlockType {
    name: "sand",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SANDSTONE: BlockType = BlockType {
    name: "sandstone",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SLATE: BlockType = BlockType {
    name: "slate",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static STEEL_BLOCK: BlockType = BlockType {
    name: "steel_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static STONE: BlockType = BlockType {
    name: "stone",
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static VERSATILIUM_BLOCK: BlockType = BlockType {
    name: "versatilium_block",
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static VOLTAGITE_BATTERY: BlockType = BlockType {
    name: "voltagite_battery",
    attributes: &[
        ("charge", AttributeType::U8(0)),
    ],
    light_emission: |block| block.attribute_value(0).expect_u8(),
    connects_to: connects_to_electricity,
    right_click: |block, _| {
        let mut block = block.clone();
        let charge = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((charge + 1) % 9));
        Some(block)
    },
    ..DEFAULTS
};
pub static VOLTAGITE_BLOCK: BlockType = BlockType {
    name: "voltagite_block",
    light_emission: light_emission_5,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
