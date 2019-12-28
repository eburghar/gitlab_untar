# General use

```
gitlab_untar 1.0
Éric BURGHARD
Extract latest projects archives from a gitlab server

USAGE:
    gitlab_untar [FLAGS] --config <config> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    More detailed output
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Configuration file containing projects and gitlab connection parameters

SUBCOMMANDS:
    get      Get and extract archives
    help     Prints this message or the help of the given subcommand(s)
    print    Print latest commit hash
```

# Print mode

For each projects specified in the config file, connect to a gitlab instance
with a given token and print the latest commit hash of a given branch.

# Get mode

```
USAGE:
    gitlab_untar --config <config> get [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -k, --keep       Skip extraction of projects if a directory with same name already exists. by default destination
                     directory is removed before extraction
    -V, --version    Prints version information

OPTIONS:
    -d, --dir <dir>                   Destination directory
    -s, --strip-components <strip>    Strip first path components of every entries in archive before extraction
                                      [default: 0]
```

For each projects specified in the config file, connect to a gitlab instance
with a given token and extract the latest archive.tar.gz of a given branch.

The config file looks like

'''yaml
host: git.mydomain.com
token: xxxxxxxxxx 
archives:
  namespace1/project1: 'branch1'
  namespace2/project2: 'branch2'
'''