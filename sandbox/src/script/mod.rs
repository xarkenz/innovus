use std::collections::HashMap;
use crate::tools::asset::AssetPool;
use crate::world::World;

pub mod builtin_commands;
pub mod utils;

pub use builtin_commands::BUILTIN_COMMANDS;

pub type CommandResult<T> = Result<T, String>;

pub type DispatchFn = fn(&[&str], &mut World, &AssetPool) -> CommandResult<String>;

#[derive(Clone, Debug)]
pub struct Command {
    name: &'static str,
    min_arg_count: usize,
    max_arg_count: usize,
    dispatch: DispatchFn,
}

impl Command {
    pub const fn new(name: &'static str, min_arg_count: usize, max_arg_count: usize, dispatch: DispatchFn) -> Self {
        Self {
            name,
            min_arg_count,
            max_arg_count,
            dispatch,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn min_arg_count(&self) -> usize {
        self.min_arg_count
    }

    pub const fn max_arg_count(&self) -> usize {
        self.max_arg_count
    }

    pub const fn accepts_arg_count(&self, arg_count: usize) -> bool {
        arg_count >= self.min_arg_count && arg_count <= self.max_arg_count
    }

    pub fn get_arg_count_string(&self) -> String {
        if self.min_arg_count == self.max_arg_count {
            self.min_arg_count.to_string()
        }
        else {
            format!("{}-{}", self.min_arg_count, self.max_arg_count)
        }
    }

    pub fn dispatch(&self, args: &[&str], world: &mut World, assets: &AssetPool) -> CommandResult<String> {
        (self.dispatch)(args, world, assets)
    }
}

pub struct ScriptingEngine {
    commands: HashMap<String, Command>,
}

impl ScriptingEngine {
    pub fn new() -> Self {
        Self {
            commands: BUILTIN_COMMANDS
                .iter()
                .map(|command| (command.name().into(), command.clone()))
                .collect(),
        }
    }

    pub fn dispatch_command(&self, command: &str, world: &mut World, assets: &AssetPool) -> CommandResult<String> {
        let command = command.strip_prefix('/').unwrap_or(command);
        let mut args = command.split_whitespace();

        let Some(command_name) = args.next().map(str::to_lowercase) else {
            return Err(assets.get_text("command.error.empty").into());
        };
        let Some(command) = self.commands.get(&command_name) else {
            return Err(assets.get_text("command.error.unknown").into());
        };

        let args = Vec::from_iter(args);
        if !command.accepts_arg_count(args.len()) {
            return Err(assets.get_template_text(
                "command.error.arg_count",
                &[&command.get_arg_count_string(), &args.len().to_string()],
            ));
        }

        command.dispatch(&args, world, assets)
    }
}
