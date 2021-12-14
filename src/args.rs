use argh::FromArgs;

#[derive(FromArgs)]
/// Extract latest projects archives from a gitlab server
pub struct Opts {
	#[argh(option, short = 'c')]
	/// configuration file containing projects and gitlab connection parameters
	pub config: String,
	#[argh(switch, short = 'v')]
	/// more detailed output
	pub verbose: bool,
	#[argh(subcommand)]
	pub subcmd: SubCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum SubCommand {
	Get(Get),
	Print(Print),
}

#[derive(FromArgs)]
/// Get and extract archives
#[argh(subcommand, name = "get")]
pub struct Get {
	#[argh(option, short = 's', default = "0")]
	/// strip first n path components of every entries in archive before extraction
	pub strip: u8,
	#[argh(option, short = 'd', default = "\"tmp\".to_string()")]
	/// destination directory
	pub dir: String,
	#[argh(switch, short = 'k')]
	/// skip extraction of projects if a directory with same name already exists. by default destination directory is removed before extraction
	pub keep: bool,
	#[argh(switch, short = 'u')]
	/// update based on packages.lock file
	pub update: bool,
}

#[derive(FromArgs)]
/// Print latest commit hash
#[argh(subcommand, name = "print")]
pub struct Print {}
