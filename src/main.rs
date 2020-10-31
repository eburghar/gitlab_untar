mod args;
mod cmd;
mod config;
mod utils;

use crate::{
    args::{Opts, SubCommand},
    cmd::{get::cmd as get, print::cmd as print},
    config::Config,
};
use anyhow::Result;

fn main() -> Result<()> {
    let opts: Opts = argh::from_env();

    // read yaml config
    let config = Config::read(&opts.config)?;

    match &opts.subcmd {
        // in get mode extract archive to specified directory
        SubCommand::Get(args) => get(&config, args, &opts),
        SubCommand::Print(_) => print(&config),
    }
}
