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
