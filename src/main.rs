#[macro_use]
extern crate clap;

use bytesize::ByteSize;
use flate2::read::GzDecoder;
use gitlab::{Gitlab, QueryParamSlice};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io;
use std::path::Path;
use tar::{Archive, EntryType};

#[derive(Clap)]
#[clap(version = "1.0", author = "Éric BURGHARD")]
struct Opts {
    #[clap(short = "c", long = "config")]
    config: String,
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "get")]
    Get(Get),
    #[clap(name = "print")]
    Print,
}

#[derive(Clap)]
struct Get {
    #[clap(short = "s", long = "strip-components", default_value = "0")]
    strip: String,
    #[clap(short = "d", long = "dir")]
    dir: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    host: String,
    token: String,
    archives: HashMap<String, String>,
}

fn main() {
    let opts = Opts::parse();

    let file = match File::open(&opts.config) {
        Ok(file) => file,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, &err),
    };

    let config: Config = match serde_yaml::from_reader(file) {
        Ok(config) => config,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, &err),
    };

    let gitlab = match Gitlab::new(&config.host, &config.token) {
        Ok(gitlab) => gitlab,
        Err(err) => panic!("error connecting to {}: {:?}", &config.host, &err),
    };

    let noparams = &[] as QueryParamSlice;
    for (prj, br) in config.archives.iter() {
        let project = match gitlab.project_by_name(&prj, noparams) {
            Ok(project) => project,
            Err(err) => {
                eprintln!("error getting project {}: {:?}", &prj, &err);
                continue;
            }
        };

        let branch = match gitlab.branch(project.id, &br, noparams) {
            Ok(branch) => branch,
            Err(err) => {
                eprintln!(
                    "error getting branch {} on project {}: {:?}",
                    &br, &prj, &err
                );
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
            SubCommand::Get(args) => {
                let dir = match &args.dir {
                    Some(dir) => {
                        let path = Path::new(dir);
                        if !path.exists() {
                            match create_dir(&path) {
                                Ok(()) => {
                                    if opts.verbose != 0 {
                                        println!("creating dir {:?}", &path);
                                    }
                                }
                                Err(err) => {
                                    panic!("error creating dir {:?}: {:?}", &path, &err);
                                }
                            }
                        };
                        path
                    }
                    None => Path::new(""),
                };
                let targz = match gitlab.get_archive(project.id, commit) {
                    Ok(archive) => archive,
                    Err(err) => {
                        eprintln!("error getting {} archive: {:?}", &project.name, &err);
                        continue;
                    }
                };

                println!("extracting branch {} of project {}", &br, &prj);
                let tar = GzDecoder::new(targz);
                let mut arquive = Archive::new(tar);
                for entry in arquive.entries().unwrap() {
                    let mut entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            eprintln!(
                                "  error getting {} arquive entry: {:?}",
                                &project.name, &err
                            );
                            continue;
                        }
                    };

                    let path = entry.path().unwrap().into_owned();
                    let mut components = path.components();
                    let strip = match args.strip.parse::<u8>() {
                        Ok(strip) => strip,
                        Err(_) => 0,
                    };
                    for _ in 0..strip {
                        components.next();
                    }
                    let dest_path = components.as_path();
                    if dest_path.to_string_lossy().is_empty() {
                        continue;
                    }
                    let dest_path = dir.join(dest_path);

                    let file_type = entry.header().entry_type();
                    match file_type {
                        EntryType::Regular => {
                            let mut file = match File::create(&dest_path) {
                                Ok(file) => file,
                                Err(err) => {
                                    eprintln!("  error creating file {:?}: {:?}", &dest_path, &err);
                                    continue;
                                }
                            };
                            match io::copy(&mut entry, &mut file) {
                                Ok(size) => {
                                    if opts.verbose != 0 {
                                        println!(
                                            "  {:?} extracted ({})",
                                            &dest_path,
                                            ByteSize(size)
                                        );
                                    }
                                }
                                Err(err) => {
                                    eprintln!("  error extracting {:?}: {:?}", &dest_path, &err);
                                    continue;
                                }
                            }
                        }
                        EntryType::Directory => match create_dir(&dest_path) {
                            Ok(()) => {
                                if opts.verbose != 0 {
                                    println!("  {:?} created", &dest_path);
                                }
                            }
                            Err(err) => {
                                eprintln!("  error creating dir {:?}: {:?}", &dest_path, &err);
                                continue;
                            }
                        },
                        _ => {
                            eprintln!("  {:?} ({:?}) ignored", &dest_path, &file_type);
                            continue;
                        }
                    }
                }
            }

            SubCommand::Print => {
                println!("{}:{}", &prj, commit.id.value());
            }
        }
    }
}
