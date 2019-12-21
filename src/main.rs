#[macro_use]
extern crate clap;

use gitlab::Gitlab;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Éric BURGHARD")]
struct Opts {
	#[clap(short = "h", long = "host")]
    host: String,
    #[clap(short = "t", long = "token")]
    token: String,
    #[clap(short = "p", long = "project")]
    project: String,
    #[clap(short = "b", long = "branch")]
    branch: String
}

fn main() {
    let opts = Opts::parse();
    let noparams = [];

    let gitlab = match Gitlab::new(&opts.host, opts.token) {
    	Ok(gitlab) => gitlab,
    	Err(err) => panic!("error connecting to {}: {:?}", opts.host, err)
    };

    let project = match gitlab.project_by_name(&opts.project, &noparams) {
    	Ok(project) => {
    		println!("project {} has id {}", &opts.project, project.id.value());
    		project
    	},
    	Err(err) => panic!("error getting project {}: {:?}", opts.project, err)
    };

    let branch = match gitlab.branch(project.id, &opts.branch, &noparams) {
        Ok(branch) => branch,
        Err(err) => panic!("error getting branch {} on project {}: {:?}", opts.branch, opts.project, err)
    };

    let commit = match branch.commit {
        Some(commit) => commit,
        None => panic!("no commit for project {}", opts.project)
    };
    println!("project {} branch {} last commit {}", opts.project, opts.branch, commit.id.value());

    let _tar = gitlab.get_with_param("repository/archive.tar.gz", [("sha", commit.id.value())]);
}
