# General use

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

# Print mode

For each projects specified in the config file, connect to a gitlab instance
with a given token and print the latest commit hash of a given branch.

# Get mode

```
Usage: gitlab_untar get [-s <strip>] [-d <dir>] [-k]

Get and extract archives

Options:
  -s, --strip       strip first path components of every entries in archive
                    before extraction
  -d, --dir         destination directory
  -k, --keep        skip extraction of projects if a directory with same name
                    already exists. by default destination directory is removed
                    before extraction
  -u, --update      update based on packages.lock
  --help            display usage information
```

For each projects specified in the config file, connect to a gitlab instance
with a given token and extract the latest archive.tar.gz of a given branch.

The config file looks like

```yaml
host: git.mydomain.com
token: xxxxxxxxxx 
archives:
  namespace1/project1: 'branch1'
  namespace2/project2: 'branch2'
```