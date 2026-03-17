use crate::script::CommandResult;
use crate::tools::asset::text::TextAsset;
use crate::world::item::{ItemType, ITEM_TYPES};

pub fn parse_u32(string: &str) -> CommandResult<u32> {
    string.parse().map_err(|_| TextAsset::template(
        "command.error.invalid_integer",
        Box::new([string.into()]),
    ).into())
}

pub fn parse_item_type(name: &str) -> CommandResult<&'static ItemType> {
    // TODO: HashMap would probably be better
    ITEM_TYPES
        .iter()
        .copied()
        .find(|item_type| item_type.name() == name)
        .ok_or_else(|| TextAsset::template(
            "command.error.no_such_item",
            Box::new([name.into()]),
        ).into())
}
