#[macro_use]
extern crate clap;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use gitlab::{Gitlab, QueryParamSlice};
use serde::{Serialize, Deserialize};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Éric BURGHARD")]
struct Opts {
	#[clap(short = "c", long = "config")]
    config: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    host: String,
    token: String,
    archives: HashMap<String, String>
}

fn main() {
    let opts = Opts::parse();

    let file = match File::open(&opts.config) {
        Err(err) => panic!("error reading {}: {:?}", &opts.config, err),
        Ok(file) => file
    };

    let config: Config = match serde_yaml::from_reader(file) {
        Err(err) => panic!("error reading {}: {:?}", &opts.config, err),
        Ok(config) => config
    };

    let gitlab = match Gitlab::new(&config.host, &config.token) {
    	Ok(gitlab) => gitlab,
    	Err(err) => panic!("error connecting to {}: {:?}", &config.host, err)
    };

    let noparams = &[] as QueryParamSlice;
    for (prj, br) in config.archives.iter() {
        let project = match gitlab.project_by_name(&prj, noparams) {
        	Ok(project) => {
        		println!("project {} has id {}", &prj, project.id.value());
        		project
        	},
        	Err(err) => {
                println!("error getting project {}: {:?}", &prj, err);
                continue;
            }
        };

        let branch = match gitlab.branch(project.id, &br, noparams) {
            Ok(branch) => branch,
            Err(err) => {
                println!("error getting branch {} on project {}: {:?}", &br, &prj, err);
                continue;
            }
        };

        let commit = match branch.commit {
            Some(commit) => commit,
            None => {
                println!("no commit for project {}", prj);
                continue;
            }
        };
        println!("project {} branch {} last commit {}", prj, br, commit.id.value());

        let mut file = File::create(format!("{}-{}.tar.gz", project.name, br)).unwrap();
        let mut archive = gitlab.get_archive(project.id, commit).unwrap();
        let _ = io::copy(&mut archive, &mut file);
    }

}
