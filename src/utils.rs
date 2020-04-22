use anyhow::{Context, Result};
use gitlab::{Gitlab, Project, QueryParamSlice, RepoCommit};
use std::collections::BTreeMap;
use std::fs::{create_dir, remove_dir_all, File};
use std::path::{Path, PathBuf};

pub struct ProjectBranch {
    pub project: Project,
    pub commit: RepoCommit,
}

pub fn get_project(gitlab: &Gitlab, prj: &str, br: &str) -> Result<ProjectBranch> {
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

pub fn get_lock(config: &str) -> Result<BTreeMap<String, String>> {
    // open lock file
    let lock = Path::new(config).with_extension("lock");
    if let Ok(file) = File::open(&lock) {
        // deserialize lock
        let commits: BTreeMap<String, String> =
            serde_yaml::from_reader(file).with_context(|| format!("Can't read {:?}", &lock))?;
        Ok(commits)
    } else {
        // create empty commits list
        let commits: BTreeMap<String, String> = BTreeMap::new();
        Ok(commits)
    }
}

pub fn save_lock(config: &str, update: bool, commits: &BTreeMap<String, String>) -> Result<()> {
    // save lock file if update mode or file doesn't exists
    let lock = Path::new(config).with_extension("lock");
    if update || !Path::new(&lock).exists() {
        if let Ok(file) = File::create(&lock) {
            serde_yaml::to_writer(file, &commits)
                .with_context(|| format!("Can't write {:?}", &lock))?;
        }
    }
    Ok(())
}

pub fn get_or_create_dir(
    dir: &String,
    keep: bool,
    update: bool,
    verbose: bool,
) -> Result<Option<PathBuf>> {
    let path = PathBuf::from(dir);
    // remove destination dir if requested
    if !keep && !update && path.exists() {
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
