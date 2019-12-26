#[macro_use]
extern crate clap;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use gitlab::{Gitlab, QueryParamSlice};
use serde::Deserialize;
use bytesize::ByteSize;

#[derive(Clap)]
#[clap(version = "1.0", author = "Éric BURGHARD")]
struct Opts {
	#[clap(short = "c", long = "config")]
    config: String,
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name="get")]
    Get,
    #[clap(name="print")]
    Print
}

#[derive(Deserialize)]
struct Config {
    host: String,
    token: String,
    archives: HashMap<String, String>
}

fn main() {
    let opts = Opts::parse();

    let file = match File::open(&opts.config) {
        Ok(file) => file,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, err)
    };

    let config: Config = match serde_yaml::from_reader(file) {
        Ok(config) => config,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, err)
    };

    let gitlab = match Gitlab::new(&config.host, &config.token) {
    	Ok(gitlab) => gitlab,
    	Err(err) => panic!("error connecting to {}: {:?}", &config.host, err)
    };

    let noparams = &[] as QueryParamSlice;
    for (prj, br) in config.archives.iter() {
        let project = match gitlab.project_by_name(&prj, noparams) {
        	Ok(project) => {
        		project
        	},
        	Err(err) => {
                eprintln!("error getting project {}: {:?}", &prj, err);
                continue;
            }
        };

        let branch = match gitlab.branch(project.id, &br, noparams) {
            Ok(branch) => branch,
            Err(err) => {
                eprintln!("error getting branch {} on project {}: {:?}", &br, &prj, err);
                continue;
            }
        };

        let commit = match branch.commit {
            Some(commit) => commit,
            None => {
                eprintln!("no commit for project {}", &prj);
                continue;
            }
        };

        match &opts.subcmd {
            SubCommand::Get => {
                let archive_name = format!("{}-{}.tar.gz", &project.name, &br);
                let mut file = match File::create(&archive_name) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("error creating {}: {:?}", &archive_name, &err);
                        continue;
                    }
                };
                let mut archive = match gitlab.get_archive(project.id, commit) {
                    Ok(archive) => archive,
                    Err(err) => {
                        eprintln!("error getting {} archive: {:?}", &archive_name, &err);
                        continue;
                    }
                };
                match io::copy(&mut archive, &mut file) {
                    Ok(size) => {
                        println!("{} downloaded ({})", &archive_name, ByteSize(size));
                    },
                    Err(err) => {
                        eprintln!("error getting {} archive: {:?}", &archive_name, &err);
                        continue;
                    }
                }
            },

            SubCommand::Print => {
                println!("{}:{}", &prj, commit.id.value());
            }
        }
    }

}