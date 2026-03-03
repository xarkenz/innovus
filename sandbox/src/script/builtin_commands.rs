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
        Some(&"max") => item_type.max_count(),
        Some(&count_text) => utils::parse_u32(count_text, assets)?.min(item_type.max_count()),
        None => 1,
    };
    let result_text;
    let item = if item_count > 0 && !item_type.is_air() {
        result_text = assets.get_template_text("command.give.success", &[
            &item_count.to_string(),
            assets.get_text(&format!("item.{item_type}")),
        ]);
        Item::new(item_type, item_count)
    } else {
        result_text = assets.get_text("command.give.success_empty").into();
        Item::default()
    };
    world.player_mut().set_held_item(item);
    Ok(result_text)
}
