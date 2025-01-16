# `find-project`: traverse through folders with ease

`find-project` is a newly updated tool that allows you to scan a given directory for folders on a depth-first basis. Simply set either the `$FP_FOLDER` environment variable (or, for backwards compatibility with my old tool, the `$GOPATH/src` folder, although only `$GOPATH` needs to be set) to the directory you want to scan, and run the tool by specifying a folder name.

`find-project` will then traverse through the directory, looking for folders that match the given name. If it finds any, it will print out the full path to the folder to `stdout` (making it suitable to be used as `bash` functions, more on this below). If it doesn't find any, it will print out an error message and exit with a non-zero status code.

By default, any `vendor` folder is skipped as well as any hidden folder (folders starting with a `.`). You can change this behavior by specifying `--include-vendor` and/or `--include-hidden` respectively.

### Example

Assume the following folder structure, where the root is either set to `$FP_FOLDER` or `$GOPATH/src`:

```plaintext
.
├── github.com/
│   ├── patrickdappollonio/
│   │   ├── http-server
│   │   ├── tgen
│   │   ├── find-project
│   │   ├── foobar
│   │   └── barbaz
│   ├── kubernetes/
│   │   ├── kubernetes
│   │   └── autoscaler
│   └── github/
│       └── octocat
└── gitlab.com/
    └── patrickdappollonio/
        ├── example1
        └── example2
```

Say you want to go from `$FP_FOLDER/github.com/patrickdappollonio/tgen` to `$FP_FOLDER/github.com/kubernetes/autoscaler`. Normally, you could do something like this (unless you have some additional aliases or functions to help you):

```bash
$ echo $FP_FOLDER
/home/patrickdap/Projects/

$ pwd
/home/patrickdap/Projects/github.com/patrickdappollonio/tgen

$ cd ../../kubernetes/autoscaler && pwd
/home/patrickdap/Projects/github.com/kubernetes/autoscaler
```

With `find-project`, you can do this instead:

```bash
$ pwd
/home/patrickdap/Projects/github.com/patrickdappollonio/tgen

$ cd $(find-project autoscaler) && pwd
/home/patrickdap/Projects/github.com/kubernetes/autoscaler
```

Guaranteeing you just need to remember the folder name you want to go to, and not the entire path.

On two competing folders with the same name, `find-project` will return the first one it finds, depth-first. For those folders with the same name, the parent folder takes precedence. The folder search is performed on a platform and OS-dependant way, unless `--sort-alphabetically` is specified (although most platforms might already sort the folders alphabetically).

### Usage

You can use the binary directly by calling `find-project <folder_name>`. The following options are available:

```plaintext
Traverse the directory specified by $FP_FOLDER or $GOPATH to find a folder depth-first.

Usage: find-project [OPTIONS] <FOLDER_NAME>

Arguments:
  <FOLDER_NAME>

Options:
      --include-vendor       Also search in "vendor" folders
      --include-hidden       Also search in hidden (dot) folders
      --sort-alphabetically  Sort folders alphabetically
  -h, --help                 Print help
  -V, --version              Print version
```

Alternatively, you can wrap the binary in a Bash function to make it easier to use. Add the following to your `.bashrc` or `.bash_profile`:

```bash
# Name your function whatever you want, I'm using "fp" for "find project".
# Older versions of this program used "gs".
function fp() {
  cd $(find-project $@)
}
```

This will allow you to run the same command as before, but without the need to type `cd` after it:

```bash
$ pwd
/home/patrickdap/Projects/github.com/patrickdappollonio/tgen

$ fp autoscaler && pwd
/home/patrickdap/Projects/github.com/kubernetes/autoscaler
```

### Installation

Download a binary from the [releases page](https://github.com/patrickdappollonio/find-project/releases) and place it in a folder that is in your `$PATH`.

Alternatively, if you're on macOS or Linux and you're a Homebrew user, you can install it via Homebrew:

```bash
brew install patrickdappollonio/tap/find-project
```
