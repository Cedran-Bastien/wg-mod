mod command;
pub mod commands;

use self::{command::RunnableCommand, commands::new::NewCommand};
use crate::cli::command::CommandError;

pub fn run() -> Result<(), CommandError> {
    let matches = command::command().get_matches();

    match matches.subcommand() {
        | Some(("new", args)) => NewCommand::run(args),
        | Some((_, _)) => Err(CommandError::CommandNotImplemented),
        | None => Err(CommandError::NoCommandProvided),
    }
}
