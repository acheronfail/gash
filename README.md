# GASH: Git hASH

Some questions:

* Do you use `git`?
* Do you think you're the bee's knees?
* Do you ever feel like your changes are lost in a sea of commits?
* Do you want to etch a `<your name> was here` forever into your `git` history?

If you answered yes to any of those questions, or if you just like what you see when you look in the mirror, then this tool is for you!

<details>
  <summary>How does it work?</summary>

  It makes your last commit contain a provided substring.
  It does this by patching the metadata of the commit, and brute forcing it until it finds a match!

  For more detail, see the genius who thought of it first:

  * https://github.com/will/git-vain
    * https://youtu.be/Jcto0Bs1hIA?t=85

</details>

## Installation

#### Precompiled binaries

See the [releases] page for pre-compiled binaries. (If CI is working, I am tired of fighting with CI.)

#### Via Cargo

```bash
cargo install gash
```

#### From Source (via Cargo)

```bash
git clone https://github.com/acheronfail/gash/
cd gash
cargo install --path .
```

## Usage

First and foremost, please see the help text:

```bash
gash --help
```

You can either run it via the command line:

```bash
cd path/to/your/git/repository
gash [options...]
```

Or, install it automatically as a git hook:

```bash
echo '#!/bin/bash\ngash [options...]' > .git/hooks/post-commit
chmod +x .git/hooks/post-commit
```

## Configuration

Most command line flags can be set in your git config, too (see `gash --help` for a comprehensive list).
For example:

```bash
# Set the default prefix to "babe":
git config --global gash.default "babe"
# Use parallel mode by default:
git config --global gash.parallel "true"

# Now, this is equivalent to running: `gash --parallel "babe"`
gash
```

[releases]: https://github.com/acheronfail/gash/releases
