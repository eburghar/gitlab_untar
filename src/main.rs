mod args;
mod cmd;
mod config;
mod utils;

use crate::args::{Opts, SubCommand};
use crate::cmd::{get::cmd as get, print::cmd as print};
use crate::config::get_config;
use crate::config::Config;
use anyhow::{Context, Result};
use gitlab::Gitlab;

fn main() -> Result<()> {
    let opts: Opts = argh::from_env();

    // get config value in a struct
    let config = get_config(&opts.config)?;
    // connect to gitlab instance using host and token from config file
    let gitlab = Gitlab::new(&config.host, &config.token)
        .with_context(|| format!("Can't connect to {}", &config.host))?;

    match &opts.subcmd {
        // in get mode extract archive to specified directory
        SubCommand::Get(_args) => get(&gitlab, &config, &opts),
        SubCommand::Print(_args) => print(&gitlab, &config, &opts),
    }
}
