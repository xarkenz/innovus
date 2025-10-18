use crate::script::CommandResult;
use crate::tools::asset::AssetPool;
use crate::world::item::{ItemType, ITEM_TYPES};

pub fn parse_u32(string: &str, assets: &AssetPool) -> CommandResult<u32> {
    string.parse().map_err(|_| assets.get_template_text(
        "command.error.invalid_integer",
        &[string],
    ))
}

pub fn parse_item_type(name: &str, assets: &AssetPool) -> CommandResult<&'static ItemType> {
    // TODO: HashMap would probably be better
    ITEM_TYPES
        .iter()
        .copied()
        .find(|item_type| item_type.name() == name)
        .ok_or_else(|| assets.get_template_text(
            "command.error.no_such_item",
            &[name],
        ))
}
