use crate::args::{Opts, SubCommand};
use crate::utils::get_project;
use crate::Config;
use anyhow::Result;
use gitlab::Gitlab;

pub fn cmd(gitlab: &Gitlab, config: &Config, opts: &Opts) -> Result<()> {
    // print project path and last commit hash
    // iterate over each project name indicated in the config file
    if let SubCommand::Print(_args) = &opts.subcmd {
        for (prj, br) in config.archives.iter() {
            let proj = get_project(&gitlab, &prj, &br)?;
            println!("{}:{}", &prj, proj.commit.id.value());
        }
    }
    Ok(())
}
