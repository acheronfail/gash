# GASH: Git hASH

## Installation

#### From Source (via Cargo)

```bash
cargo install gash
```

## Usage

First and foremost, please see the help text:

```bash
gash --help
```

You can either run it via the command line:

```bash
cd path/to/your/git/repository
# Amend the last commit to start with "cafe":
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
