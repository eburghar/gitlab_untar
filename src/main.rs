#[macro_use]
extern crate clap;

use bytesize::ByteSize;
use flate2::read::GzDecoder;
use gitlab::{Gitlab, QueryParamSlice};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all, File};
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
    #[clap(short = "k", long = "clean", parse(from_occurrences))]
    clean: u8,
}

#[derive(Deserialize)]
struct Config {
    host: String,
    token: String,
    archives: HashMap<String, String>,
}

fn main() {
    let opts = Opts::parse();

    // open configuration file
    let file = match File::open(&opts.config) {
        Ok(file) => file,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, &err),
    };

    // deserialize configuration
    let config: Config = match serde_yaml::from_reader(file) {
        Ok(config) => config,
        Err(err) => panic!("error reading {}: {:?}", &opts.config, &err),
    };

    // connect to gitlab instance using host and token from config file
    let gitlab = match Gitlab::new(&config.host, &config.token) {
        Ok(gitlab) => gitlab,
        Err(err) => panic!("error connecting to {}: {:?}", &config.host, &err),
    };

    // create the dest directory if using get subcommand
    // and save as an Option<Path> for later use
    let dest_dir = match &opts.subcmd {
        SubCommand::Get(args) => match &args.dir {
            Some(dir) => {
                let path = Path::new(dir);
                // remove destination dir if requested
                if args.clean > 0 {
                    if path.exists() {
                        match remove_dir_all(&path) {
                            Ok(()) => {
                                if opts.verbose != 0 {
                                    println!("{} removed", &dir)
                                }
                            }
                            Err(err) => panic!("error removing {}: {:?}", &dir, &err),
                        }
                    }
                }
                // create destination dir if necessary
                if !path.exists() {
                    match create_dir(&path) {
                        Ok(()) => {
                            if opts.verbose != 0 {
                                println!("creating dir {}", &dir);
                            }
                        }
                        Err(err) => panic!("error creating dir {}: {:?}", &dir, &err),
                    }
                }
                Some(path)
            }
            None => Some(Path::new("")),
        },
        _ => None,
    };

    let noparams = &[] as QueryParamSlice;
    // iterate over each project name indicated in the config file
    for (prj, br) in config.archives.iter() {
        // get project definition from project name
        let project = match gitlab.project_by_name(&prj, noparams) {
            Ok(project) => project,
            Err(err) => {
                eprintln!("error getting project {}: {:?}", &prj, &err);
                continue;
            }
        };

        // get indicated branch
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

        // get last commmit of the branch
        let commit = match branch.commit {
            Some(commit) => commit,
            None => {
                eprintln!("no commit for project {}", &prj);
                continue;
            }
        };

        match &opts.subcmd {
            // in get mode extract archive to specified directory
            SubCommand::Get(args) => {
                // get the archive.tar.gz from project branch last commit
                let targz = match gitlab.get_archive(project.id, commit) {
                    Ok(archive) => archive,
                    Err(err) => {
                        eprintln!("error getting {} archive: {:?}", &project.name, &err);
                        continue;
                    }
                };

                println!("extracting branch {} of {}", &br, &prj);
                // chain gzip reader and arquive reader
                let tar = GzDecoder::new(targz);
                let mut arquive = Archive::new(tar);
                // for each entry in the arquive
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

                    // get the path
                    let mut entry_path = entry.path().unwrap().into_owned();
                    // turn into components
                    let mut components = entry_path.components();
                    // remove first components if indicated in command line args
                    if let Ok(strip) = args.strip.parse::<u8>() {
                        if strip > 0 {
                            for _ in 0..strip {
                                components.next();
                            }
                            // and reassemble dest_path
                            entry_path = components.as_path().to_path_buf();
                        }
                    };
                    // don't do anything if empty path
                    if entry_path.to_string_lossy().is_empty() {
                        continue;
                    }
                    // append destination dir to entry path
                    entry_path = dest_dir.unwrap().join(entry_path);

                    // get the entry type
                    let file_type = entry.header().entry_type();
                    match file_type {
                        // if it's a file, extract it to local filesystem
                        EntryType::Regular => {
                            let mut file = match File::create(&entry_path) {
                                Ok(file) => file,
                                Err(err) => {
                                    eprintln!(
                                        "  error creating file {}: {:?}",
                                        &entry_path.to_string_lossy(),
                                        &err
                                    );
                                    continue;
                                }
                            };
                            match io::copy(&mut entry, &mut file) {
                                Ok(size) => {
                                    if opts.verbose != 0 {
                                        println!(
                                            "  {} ({})",
                                            &entry_path.to_string_lossy(),
                                            ByteSize(size)
                                        );
                                    }
                                }
                                Err(err) => {
                                    eprintln!(
                                        "  error extracting {}: {:?}",
                                        &entry_path.to_string_lossy(),
                                        &err
                                    );
                                    continue;
                                }
                            }
                        }
                        // if it's a directory, create it
                        EntryType::Directory => match create_dir(&entry_path) {
                            Ok(()) => {
                                if opts.verbose != 0 {
                                    println!("  {}", &entry_path.to_string_lossy());
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "  error creating dir {}: {:?}",
                                    &entry_path.to_string_lossy(),
                                    &err
                                );
                                continue;
                            }
                        },
                        // TODO: support other types (links)
                        _ => {
                            eprintln!(
                                "  {} ({:?}) ignored",
                                &entry_path.to_string_lossy(),
                                &file_type
                            );
                            continue;
                        }
                    }
                }
            }

            // if print mode just print project path and last commit hash
            SubCommand::Print => {
                println!("{}:{}", &prj, commit.id.value());
            }
        }
    }
}
