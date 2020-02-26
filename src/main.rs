use anyhow::{Context, Result};
use argh::FromArgs;
use bytesize::ByteSize;
use flate2::read::GzDecoder;
use gitlab::{Gitlab, Project, QueryParamSlice, RepoCommit};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all, File};
use std::io;
use std::path::PathBuf;
use tar::{Archive, EntryType};

#[derive(FromArgs)]
/// Extract latest projects archives from a gitlab server
struct Opts {
    #[argh(option, short = 'c')]
    /// configuration file containing projects and gitlab connection parameters
    config: String,
    #[argh(switch, short = 'v')]
    /// more detailed output
    verbose: bool,
    #[argh(subcommand)]
    subcmd: SubCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Get(Get),
    Print(Print),
}

#[derive(FromArgs)]
/// Get and extract archives
#[argh(subcommand, name = "get")]
struct Get {
    #[argh(option, short = 's', default = "0")]
    /// strip first path components of every entries in archive before extraction
    strip: u8,
    #[argh(option, short = 'd', default = "\"tmp\".to_string()")]
    /// destination directory
    dir: String,
    #[argh(switch, short = 'k')]
    /// skip extraction of projects if a directory with same name already exists. by default destination directory is removed before extraction
    keep: bool,
}

#[derive(FromArgs)]
/// Print latest commit hash
#[argh(subcommand, name = "print")]
struct Print {}

#[derive(Deserialize)]
struct Config {
    host: String,
    token: String,
    archives: HashMap<String, String>,
}

struct ProjectBranch {
    project: Project,
    commit: RepoCommit,
}

fn get_project(gitlab: &Gitlab, prj: &str, br: &str) -> Result<ProjectBranch> {
    let noparams = &[] as QueryParamSlice;
    // get project definition from project name
    let project = gitlab
        .project_by_name(&prj, noparams)
        .with_context(|| format!("Can't get project {}", &prj))?;
    // get indicated branch
    let branch = gitlab
        .branch(project.id, &br, noparams)
        .with_context(|| format!("Can't get branch {} of project {}", &br, &prj))?;
    // get last commmit of the branch
    let commit = branch
        .commit
        .with_context(|| format!("No commit for project {}", &prj))?;

    Ok(ProjectBranch { project, commit })
}

fn get_config(config: &str) -> Result<Config> {
    // open configuration file
    let file = File::open(&config).with_context(|| format!("Can't open {}", &config))?;
    // deserialize configuration
    let config: Config =
        serde_yaml::from_reader(file).with_context(|| format!("Can't read {}", &config))?;
    Ok(config)
}

fn get_or_create_dir(dir: &String, keep: bool, verbose: bool) -> Result<Option<PathBuf>> {
    let path = PathBuf::from(dir);
    // remove destination dir if requested
    if !keep && path.exists() {
        remove_dir_all(&path).with_context(|| format!("Can't remove dir {}", &dir))?;
        if verbose {
            println!("{} removed", &dir)
        }
    }
    // create destination dir if necessary
    if !path.exists() {
        create_dir(&path).with_context(|| format!("Can't create dir {}", &dir))?;
        if verbose {
            println!("creating dir {}", &dir);
        }
    }
    Ok(Some(path))
}

fn cmd_get(gitlab: &Gitlab, config: &Config, opts: &Opts) -> Result<()> {
    if let SubCommand::Get(args) = &opts.subcmd {
        // create the dest directory and save as an Option<Path> for later use
        let dest_dir = get_or_create_dir(&args.dir, args.keep, opts.verbose)?.unwrap();

        // in get modextract archive to specified directory
        // iterate over each project name indicated in the config file
        for (prj, br) in config.archives.iter() {
            // skip gitlab requests and extraction if a dir with the name of the project already exists
            let i = match prj.rfind('/') {
                Some(i) if (i + 1) < prj.len() => i + 1,
                _ => 0,
            };
            if args.keep && dest_dir.join(&prj[i..]).exists() {
                println!("{} already extracted", &prj);
                continue;
            }

            let proj = match get_project(&gitlab, &prj, &br) {
                Ok(proj) => proj,
                Err(err) => {
                    eprintln!("{}", &err);
                    continue;
                }
            };

            let project = proj.project;
            let commit = proj.commit;

            // get the archive.tar.gz from project branch last commit
            let targz = match gitlab.get_archive(project.id, commit) {
                Ok(archive) => archive,
                Err(err) => {
                    eprintln!("Can't get {} archive: {:?}", &project.name, &err);
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
                        eprintln!("  Can't get {} arquive entry: {:?}", &project.name, &err);
                        continue;
                    }
                };

                // get the path
                let mut entry_path = entry.path().unwrap().into_owned();
                // turn into components
                let mut components = entry_path.components();
                // skip first components if indicated in command line args
                if args.strip > 0 {
                    for _ in 0..args.strip {
                        components.next();
                    }
                    // and reassemble dest_path
                    entry_path = components.as_path().to_path_buf();
                }
                // don't do anything if empty path
                if entry_path.to_string_lossy().is_empty() {
                    continue;
                }
                // append destination dir to entry path
                entry_path = dest_dir.join(entry_path);
                // get the entry type
                let file_type = entry.header().entry_type();
                match file_type {
                    // if it's a directory, create it if doesn't exist
                    EntryType::Directory => {
                        if !entry_path.exists() {
                            match create_dir(&entry_path) {
                                Ok(()) => {
                                    if opts.verbose {
                                        println!("  {}", &entry_path.to_string_lossy());
                                    }
                                }
                                Err(err) => {
                                    eprintln!(
                                        "  Can't create dir {}: {:?}",
                                        &entry_path.to_string_lossy(),
                                        &err
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    // if it's a file, extract it to local filesystem
                    EntryType::Regular => {
                        let mut file = match File::create(&entry_path) {
                            Ok(file) => file,
                            Err(err) => {
                                eprintln!(
                                    "  Can't create file {}: {:?}",
                                    &entry_path.to_string_lossy(),
                                    &err
                                );
                                continue;
                            }
                        };
                        match io::copy(&mut entry, &mut file) {
                            Ok(size) => {
                                if opts.verbose {
                                    println!(
                                        "  {} ({})",
                                        &entry_path.to_string_lossy(),
                                        ByteSize(size)
                                    );
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "  Can't extract {}: {:?}",
                                    &entry_path.to_string_lossy(),
                                    &err
                                );
                                continue;
                            }
                        }
                    }
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
    }

    Ok(())
}

fn cmd_print(gitlab: &Gitlab, config: &Config, opts: &Opts) -> Result<()> {
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

fn main() -> Result<()> {
    let opts: Opts = argh::from_env();

    // get config value in a struct
    let config = get_config(&opts.config)?;
    // connect to gitlab instance using host and token from config file
    let gitlab = Gitlab::new(&config.host, &config.token)
        .with_context(|| format!("Can't connect to {}", &config.host))?;

    match &opts.subcmd {
        // in get mode extract archive to specified directory
        SubCommand::Get(_args) => cmd_get(&gitlab, &config, &opts),
        SubCommand::Print(_args) => cmd_print(&gitlab, &config, &opts),
    }
}
