# gitlab_untar

gitlab_untar allows you to get and extract projects archives from a gitlab instance whithout having to install git
or a shell or any toolchain (npm, pypi, ...), greatly reducing the surface attack and the execution speed. It's
specially usefull in containers that needs to specialize (really quickly) at initialization and extract a defined
set of plugins. We use that tool for odoo container.

## General use

```
Usage: gitlab_untar -c <config> [-v] <command> [<args>]

Extract latest projects archives from a gitlab server

Options:
  -c, --config      configuration file containing projects and gitlab connection
                    parameters
  -v, --verbose     more detailed output
  --help            display usage information

Commands:
  get               Get and extract archives
  print             Print latest commit hash
```

## Print mode

```
Usage: gitlab_untar print

Print latest commit hash

Options:
  --help            display usage information
```

For each projects specified in the config file, connect to a gitlab instance
with a given token and print the latest commit hash of a given branch.

## Get mode

```
Usage: gitlab_untar get [-s <strip>] [-d <dir>] [-k] [-u]

Get and extract archives

Options:
  -s, --strip       strip first n path components of every entries in archive
                    before extraction
  -d, --dir         destination directory
  -k, --keep        skip extraction of projects if a directory with same name
                    already exists. by default destination directory is removed
                    before extraction
  -u, --update      update based on packages.lock file
  --help            display usage information
```

For each projects specified in the config file, connect to a gitlab instance with a given token and extract the
latest archive.tar.gz of a given branch. The extraction is done from the stream whithout needing to preliminary
download and save the archive on disk.

In update mode, a lock file containing hash of latest commit is used to decide if we need to extract again archives

## Configuration

The config file looks like

```yaml
host: git.mydomain.com
token: xxxxxxxxxx 
archives:
  namespace1/project1: 'branch1'
  namespace2/project2: 'branch2'
```

The token is a regular gitlab access token with read privilege.
