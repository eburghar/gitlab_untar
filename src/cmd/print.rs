use crate::{utils::get_project, Config};
use anyhow::{Context, Result};
use gitlab::Gitlab;

pub fn cmd(config: &Config) -> Result<()> {
	// connect to gitlab instance using host and token from config file
	let gitlab = Gitlab::new(&config.host, &config.token)
		.with_context(|| format!("Can't connect to {}", &config.host))?;

	// print project path and last commit hash
	// iterate over each project name indicated in the config file
	for (prj, br) in config.archives.iter() {
		let proj = get_project(&gitlab, prj, br)?;
		println!("{}:{}", &prj, proj.commit.id.value());
	}
	Ok(())
}
