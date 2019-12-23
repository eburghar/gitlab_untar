#[macro_use]
extern crate clap;

use std::fs::File;
use std::io;
use gitlab::{Gitlab, QueryParamSlice};

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
    let noparams = &[] as QueryParamSlice;

    let gitlab = match Gitlab::new(&opts.host, opts.token) {
    	Ok(gitlab) => gitlab,
    	Err(err) => panic!("error connecting to {}: {:?}", opts.host, err)
    };

    let project = match gitlab.project_by_name(&opts.project, noparams) {
    	Ok(project) => {
    		println!("project {} has id {}", &opts.project, project.id.value());
    		project
    	},
    	Err(err) => panic!("error getting project {}: {:?}", opts.project, err)
    };

    let branch = match gitlab.branch(project.id, &opts.branch, noparams) {
        Ok(branch) => branch,
        Err(err) => panic!("error getting branch {} on project {}: {:?}", opts.branch, opts.project, err)
    };

    let commit = match branch.commit {
        Some(commit) => commit,
        None => panic!("no commit for project {}", opts.project)
    };
    println!("project {} branch {} last commit {}", opts.project, opts.branch, commit.id.value());

    let mut file = File::create("archive.tar.gz").unwrap();
    let mut archive = gitlab.get_archive(project.id, commit).unwrap();
    let _ = io::copy(&mut archive, &mut file);
}
