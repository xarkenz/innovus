use crate::world::block;
use super::*;

fn right_click_no_action(target_block: &Block, held_item: &Item) -> (Option<Block>, Option<Item>) {
    let _ = (target_block, held_item);
    (None, None)
}

fn right_click_place_block(target_block: &Block, held_item: &Item) -> (Option<Block>, Option<Item>) {
    if held_item.count() > 0 && target_block.block_type() == &block::types::AIR {
        let block_type = held_item.item_type().block_type().unwrap();
        (Some(Block::new(block_type)), Some(held_item.decrement_count()))
    }
    else {
        (None, None)
    }
}

const DEFAULTS: ItemType = ItemType {
    name: "invalid",
    max_count: 100,
    block_type: None,
    right_click: right_click_no_action,
};

pub const ITEM_TYPES: &[&ItemType] = &[
    &AIR,
    &ALUMINUM,
    &ALUMINUM_AXE,
    &ALUMINUM_BLOCK,
    &ALUMINUM_PICKAXE,
    &ALUMINUM_SHOVEL,
    &ALUMINUM_SWORD,
    &AMETHYST,
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
    &OAK_WOOD,
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

pub static AIR: ItemType = ItemType {
    name: "air",
    max_count: 0,
    ..DEFAULTS
};
pub static ALUMINUM: ItemType = ItemType {
    name: "aluminum",
    ..DEFAULTS
};
pub static ALUMINUM_AXE: ItemType = ItemType {
    name: "aluminum_axe",
    max_count: 1,
    ..DEFAULTS
};
pub static ALUMINUM_BLOCK: ItemType = ItemType {
    name: "aluminum_block",
    block_type: Some(&block::types::ALUMINUM_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static ALUMINUM_PICKAXE: ItemType = ItemType {
    name: "aluminum_pickaxe",
    max_count: 1,
    ..DEFAULTS
};
pub static ALUMINUM_SHOVEL: ItemType = ItemType {
    name: "aluminum_shovel",
    max_count: 1,
    ..DEFAULTS
};
pub static ALUMINUM_SWORD: ItemType = ItemType {
    name: "aluminum_sword",
    max_count: 1,
    ..DEFAULTS
};
pub static AMETHYST: ItemType = ItemType {
    name: "amethyst",
    ..DEFAULTS
};
pub static AMETHYST_BLOCK: ItemType = ItemType {
    name: "amethyst_block",
    block_type: Some(&block::types::AMETHYST_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static AMETHYST_CRYSTAL: ItemType = ItemType {
    name: "amethyst_crystal",
    block_type: Some(&block::types::AMETHYST_CRYSTAL),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static AMETHYST_ORE: ItemType = ItemType {
    name: "amethyst_ore",
    block_type: Some(&block::types::AMETHYST_ORE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static AMPLIFITE_BLOCK: ItemType = ItemType {
    name: "amplifite_block",
    block_type: Some(&block::types::AMPLIFITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static CHAIN: ItemType = ItemType {
    name: "chain",
    block_type: Some(&block::types::CHAIN),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static COAL_BLOCK: ItemType = ItemType {
    name: "coal_block",
    block_type: Some(&block::types::COAL_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static COBALT_BLOCK: ItemType = ItemType {
    name: "cobalt_block",
    block_type: Some(&block::types::COBALT_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static COBBLES: ItemType = ItemType {
    name: "cobbles",
    block_type: Some(&block::types::COBBLES),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static COPPER_BLOCK: ItemType = ItemType {
    name: "copper_block",
    block_type: Some(&block::types::COPPER_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static COPPER_WIRE: ItemType = ItemType {
    name: "copper_wire",
    block_type: Some(&block::types::COPPER_WIRE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static CORRUPTITE_BLOCK: ItemType = ItemType {
    name: "corruptite_block",
    block_type: Some(&block::types::CORRUPTITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static DIAMOND_BLOCK: ItemType = ItemType {
    name: "diamond_block",
    block_type: Some(&block::types::DIAMOND_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static DIRT: ItemType = ItemType {
    name: "dirt",
    block_type: Some(&block::types::DIRT),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static EMERALD_BLOCK: ItemType = ItemType {
    name: "emerald_block",
    block_type: Some(&block::types::EMERALD_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static FLAMARITE_BLOCK: ItemType = ItemType {
    name: "flamarite_block",
    block_type: Some(&block::types::FLAMARITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static FRIGIDITE_BLOCK: ItemType = ItemType {
    name: "frigidite_block",
    block_type: Some(&block::types::FRIGIDITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static GLASS: ItemType = ItemType {
    name: "glass",
    block_type: Some(&block::types::GLASS),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static GOLD_BLOCK: ItemType = ItemType {
    name: "gold_block",
    block_type: Some(&block::types::GOLD_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static GOLD_WIRE: ItemType = ItemType {
    name: "gold_wire",
    block_type: Some(&block::types::GOLD_WIRE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static GRASSY_DIRT: ItemType = ItemType {
    name: "grassy_dirt",
    block_type: Some(&block::types::GRASSY_DIRT),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static IRON_BLOCK: ItemType = ItemType {
    name: "iron_block",
    block_type: Some(&block::types::IRON_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static LANTERN: ItemType = ItemType {
    name: "lantern",
    block_type: Some(&block::types::LANTERN),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static LUMINITE_BLOCK: ItemType = ItemType {
    name: "luminite_block",
    block_type: Some(&block::types::LUMINITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static MAGMIUM_BLOCK: ItemType = ItemType {
    name: "magmium_block",
    block_type: Some(&block::types::MAGMIUM_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static OAK_WOOD: ItemType = ItemType {
    name: "oak_wood",
    block_type: Some(&block::types::OAK_WOOD),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static OBSIDIAN_BLOCK: ItemType = ItemType {
    name: "obsidian_block",
    block_type: Some(&block::types::OBSIDIAN_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static PHYLUMUS_BLOCK: ItemType = ItemType {
    name: "phylumus_block",
    block_type: Some(&block::types::PHYLUMUS_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static PHYLUMUS_MUSHROOM: ItemType = ItemType {
    name: "phylumus_mushroom",
    block_type: Some(&block::types::PHYLUMUS_MUSHROOM),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static PIPE: ItemType = ItemType {
    name: "pipe",
    block_type: Some(&block::types::PIPE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static PIPE_SPOUT: ItemType = ItemType {
    name: "pipe_spout",
    block_type: Some(&block::types::PIPE_SPOUT),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static QUARTZ_BLOCK: ItemType = ItemType {
    name: "quartz_block",
    block_type: Some(&block::types::QUARTZ_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static QUARTZ_CRYSTAL: ItemType = ItemType {
    name: "quartz_crystal",
    block_type: Some(&block::types::QUARTZ_CRYSTAL),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static QUARTZ_ORE: ItemType = ItemType {
    name: "quartz_ore",
    block_type: Some(&block::types::QUARTZ_ORE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static SAND: ItemType = ItemType {
    name: "sand",
    block_type: Some(&block::types::SAND),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static SANDSTONE: ItemType = ItemType {
    name: "sandstone",
    block_type: Some(&block::types::SANDSTONE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static SLATE: ItemType = ItemType {
    name: "slate",
    block_type: Some(&block::types::SLATE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static STEEL_BLOCK: ItemType = ItemType {
    name: "steel_block",
    block_type: Some(&block::types::STEEL_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static STONE: ItemType = ItemType {
    name: "stone",
    block_type: Some(&block::types::STONE),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static VERSATILIUM_BLOCK: ItemType = ItemType {
    name: "versatilium_block",
    block_type: Some(&block::types::VERSATILIUM_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static VOLTAGITE_BATTERY: ItemType = ItemType {
    name: "voltagite_battery",
    block_type: Some(&block::types::VOLTAGITE_BATTERY),
    right_click: right_click_place_block,
    ..DEFAULTS
};
pub static VOLTAGITE_BLOCK: ItemType = ItemType {
    name: "voltagite_block",
    block_type: Some(&block::types::VOLTAGITE_BLOCK),
    right_click: right_click_place_block,
    ..DEFAULTS
};
