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

fn connects_to_pipe(this: &Block, that: &Block) -> bool {
    let _ = this;
    that.block_type() == &PIPE || that.block_type() == &PIPE_SPOUT
}

fn right_click_no_action(this: &Block, hand: &'static BlockType) -> Option<Block> {
    let _ = (this, hand);
    None
}

const DEFAULTS: BlockType = BlockType {
    name: "invalid",
    attributes: &[],
    colliders: &[BLOCK_RECT],
    palette_key: None,
    is_full_block: full_block_always,
    light_emission: light_emission_0,
    connects_to: connects_never,
    right_click: right_click_no_action,
};

pub const BLOCK_TYPES: &[&BlockType] = &[
    &AIR,
    &TEST_BLOCK,
    &ALUMINUM_BLOCK,
    &AMETHYST_BLOCK,
    &AMETHYST_CRYSTAL,
    &AMETHYST_ORE,
    &AMPLIFITE_BLOCK,
    &CHAIN,
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
    &LANTERN,
    &LUMINITE_BLOCK,
    &MAGMIUM_BLOCK,
    &OBSIDIAN_BLOCK,
    &PHYLUMUS_BLOCK,
    &PHYLUMUS_MUSHROOM,
    &PIPE,
    &PIPE_SPOUT,
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
pub static TEST_BLOCK: BlockType = BlockType {
    name: "test_block",
    ..DEFAULTS
};
pub static ALUMINUM_BLOCK: BlockType = BlockType {
    name: "aluminum_block",
    palette_key: Some("aluminum"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static AMETHYST_BLOCK: BlockType = BlockType {
    name: "amethyst_block",
    palette_key: Some("amethyst"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static AMETHYST_CRYSTAL: BlockType = BlockType {
    name: "amethyst_crystal",
    attributes: &[
        ("wall", AttributeType::Enum { default_value: 0, value_names: &["bottom", "left", "right", "top"] }),
    ],
    colliders: &[],
    palette_key: Some("amethyst"),
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
    palette_key: Some("amethyst"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static AMPLIFITE_BLOCK: BlockType = BlockType {
    name: "amplifite_block",
    palette_key: Some("amplifite"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static CHAIN: BlockType = BlockType {
    name: "chain",
    attributes: &[
        ("axis", AttributeType::Enum { default_value: 0, value_names: &["x", "y"] }),
    ],
    colliders: &[],
    palette_key: Some("iron"),
    is_full_block: full_block_never,
    right_click: |block, _| {
        let mut block = block.clone();
        let axis = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((axis + 1) % 2));
        Some(block)
    },
    ..DEFAULTS
};
pub static COAL_BLOCK: BlockType = BlockType {
    name: "coal_block",
    palette_key: Some("coal"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COBALT_BLOCK: BlockType = BlockType {
    name: "cobalt_block",
    palette_key: Some("cobalt"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COBBLES: BlockType = BlockType {
    name: "cobbles",
    palette_key: Some("stone"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static COPPER_BLOCK: BlockType = BlockType {
    name: "copper_block",
    palette_key: Some("copper"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static COPPER_WIRE: BlockType = BlockType {
    name: "copper_wire",
    colliders: &[],
    palette_key: Some("copper"),
    is_full_block: full_block_never,
    connects_to: connects_to_electricity,
    ..DEFAULTS
};
pub static CORRUPTITE_BLOCK: BlockType = BlockType {
    name: "corruptite_block",
    palette_key: Some("corruptite"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static DIAMOND_BLOCK: BlockType = BlockType {
    name: "diamond_block",
    palette_key: Some("diamond"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static DIRT: BlockType = BlockType {
    name: "dirt",
    palette_key: Some("dirt"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static EMERALD_BLOCK: BlockType = BlockType {
    name: "emerald_block",
    palette_key: Some("emerald"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static FLAMARITE_BLOCK: BlockType = BlockType {
    name: "flamarite_block",
    palette_key: Some("flamarite"),
    light_emission: light_emission_5,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static FRIGIDITE_BLOCK: BlockType = BlockType {
    name: "frigidite_block",
    palette_key: Some("frigidite"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GLASS: BlockType = BlockType {
    name: "glass",
    palette_key: Some("glass"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GOLD_BLOCK: BlockType = BlockType {
    name: "gold_block",
    palette_key: Some("gold"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static GOLD_WIRE: BlockType = BlockType {
    name: "gold_wire",
    colliders: &[],
    palette_key: Some("gold"),
    is_full_block: full_block_never,
    connects_to: connects_to_electricity,
    ..DEFAULTS
};
pub static GRASSY_DIRT: BlockType = BlockType {
    name: "grassy_dirt",
    palette_key: Some("dirt"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static IRON_BLOCK: BlockType = BlockType {
    name: "iron_block",
    palette_key: Some("iron"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static LANTERN: BlockType = BlockType {
    name: "lantern",
    attributes: &[
        ("type", AttributeType::Enum { default_value: 0, value_names: &["floor", "hanging", "left", "right"] }),
    ],
    colliders: &[],
    palette_key: Some("iron"),
    is_full_block: full_block_never,
    light_emission: light_emission_15,
    right_click: |block, _| {
        let mut block = block.clone();
        let type_ = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((type_ + 1) % 4));
        Some(block)
    },
    ..DEFAULTS
};
pub static LUMINITE_BLOCK: BlockType = BlockType {
    name: "luminite_block",
    palette_key: Some("luminite"),
    light_emission: light_emission_15,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static MAGMIUM_BLOCK: BlockType = BlockType {
    name: "magmium_block",
    palette_key: Some("magmium"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static OBSIDIAN_BLOCK: BlockType = BlockType {
    name: "obsidian_block",
    palette_key: Some("obsidian"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static PHYLUMUS_BLOCK: BlockType = BlockType {
    name: "phylumus_block",
    palette_key: Some("phylumus"),
    light_emission: light_emission_15,
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static PHYLUMUS_MUSHROOM: BlockType = BlockType {
    name: "phylumus_mushroom",
    attributes: &[
        ("size", AttributeType::Enum { default_value: 0, value_names: &["large", "small"] }),
    ],
    colliders: &[],
    palette_key: Some("phylumus"),
    is_full_block: full_block_never,
    light_emission: |block| {
        let shape = block.attribute_value(0).expect_u8();
        if shape == 0 { 6 } else { 3 }
    },
    right_click: |block, _| {
        let mut block = block.clone();
        let shape = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((shape + 1) % 2));
        Some(block)
    },
    ..DEFAULTS
};
pub static PIPE: BlockType = BlockType {
    name: "pipe",
    colliders: &[],
    palette_key: Some("aluminum"),
    is_full_block: full_block_never,
    connects_to: connects_to_pipe,
    ..DEFAULTS
};
pub static PIPE_SPOUT: BlockType = BlockType {
    name: "pipe_spout",
    attributes: &[
        ("direction", AttributeType::Enum { default_value: 0, value_names: &["down", "left", "right", "up"] }),
    ],
    colliders: &[],
    palette_key: Some("aluminum"),
    is_full_block: full_block_never,
    right_click: |block, _| {
        let mut block = block.clone();
        let direction = block.attribute_value(0).expect_u8();
        block.set_attribute_value(0, AttributeValue::U8((direction + 1) % 4));
        Some(block)
    },
    ..DEFAULTS
};
pub static QUARTZ_BLOCK: BlockType = BlockType {
    name: "quartz_block",
    palette_key: Some("quartz"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static QUARTZ_CRYSTAL: BlockType = BlockType {
    name: "quartz_crystal",
    attributes: &[
        ("wall", AttributeType::Enum { default_value: 0, value_names: &["bottom", "left", "right", "top"] }),
    ],
    colliders: &[],
    palette_key: Some("quartz"),
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
    palette_key: Some("quartz"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SAND: BlockType = BlockType {
    name: "sand",
    palette_key: Some("sand"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SANDSTONE: BlockType = BlockType {
    name: "sandstone",
    palette_key: Some("sand"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static SLATE: BlockType = BlockType {
    name: "slate",
    palette_key: Some("slate"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static STEEL_BLOCK: BlockType = BlockType {
    name: "steel_block",
    palette_key: Some("steel"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static STONE: BlockType = BlockType {
    name: "stone",
    palette_key: Some("stone"),
    connects_to: connects_to_full_block,
    ..DEFAULTS
};
pub static VERSATILIUM_BLOCK: BlockType = BlockType {
    name: "versatilium_block",
    palette_key: Some("versatilium"),
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
pub static VOLTAGITE_BATTERY: BlockType = BlockType {
    name: "voltagite_battery",
    attributes: &[
        ("charge", AttributeType::U8(0)),
    ],
    palette_key: Some("voltagite"),
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
    palette_key: Some("voltagite"),
    light_emission: light_emission_5,
    connects_to: connects_to_same_type,
    ..DEFAULTS
};
