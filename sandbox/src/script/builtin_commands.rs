use crate::script::{Command, CommandResult, utils};
use crate::tools::asset::text::TextAsset;
use crate::world::item::Item;
use crate::world::World;

pub const BUILTIN_COMMANDS: &[Command] = &[
    Command::new("hello", 0, 0, hello),
    Command::new("give", 1, 2, give),
];

pub fn hello(args: &[&str], world: &mut World) -> CommandResult {
    let _ = (args, world);
    Ok(TextAsset::simple("command.hello").into())
}

pub fn give(args: &[&str], world: &mut World) -> CommandResult {
    let item_type = utils::parse_item_type(args[0])?;
    let item_count = match args.get(1) {
        Some(&"max") => item_type.max_count(),
        Some(&count_text) => utils::parse_u32(count_text)?.min(item_type.max_count()),
        None => 1,
    };
    let result_text;
    let item = if item_count > 0 && !item_type.is_air() {
        result_text = TextAsset::template("command.give.success", Box::new([
            item_count.to_string().into(),
            TextAsset::simple(format!("item.{item_type}")).into(),
        ])).into();
        Item::new(item_type, item_count)
    } else {
        result_text = TextAsset::simple("command.give.success_empty").into();
        Item::default()
    };
    world.player_mut().set_held_item(item);
    Ok(result_text)
}
