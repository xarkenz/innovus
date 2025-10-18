use crate::script::{Command, CommandResult, utils};
use crate::tools::asset::AssetPool;
use crate::world::item::Item;
use crate::world::World;

pub const BUILTIN_COMMANDS: &[Command] = &[
    Command::new("hello", 0, 0, hello),
    Command::new("give", 1, 2, give),
];

pub fn hello(args: &[&str], world: &mut World, assets: &AssetPool) -> CommandResult<String> {
    let _ = (args, world, assets);
    Ok("Hello, world!".into())
}

pub fn give(args: &[&str], world: &mut World, assets: &AssetPool) -> CommandResult<String> {
    let item_type = utils::parse_item_type(args[0], assets)?;
    let item_count = match args.get(1) {
        Some(count_text) => utils::parse_u32(count_text, assets)?,
        None => 1,
    };
    let item = if item_count > 0 && !item_type.is_air() {
        Item::new(item_type, item_count)
    } else {
        Item::default()
    };
    world.player_mut().set_held_item(item);
    Ok(assets.get_text("command.success").into())
}
