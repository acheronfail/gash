# GASH: Git hASH

## Installation

#### Precompiled binaries

See the [releases] page for pre-compiled binaries.

#### From Source (via Cargo)

Until `clap v3` is published, you'll have to install this package locally.
This crate can't be published until `clap v3` is published too.

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
