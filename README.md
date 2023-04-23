#  Gossip Glomers

![Lint and Deploy docs](https://github.com/sak96/gossip_glomers/actions/workflows/main.yml/badge.svg)
![License](https://img.shields.io/github/license/sak96/gossip_glomers)
![Top Language](https://img.shields.io/github/languages/top/sak96/gossip_glomers)

`Gossip Glomers` is series of distributed systems challenges.
More details of these challenges can be found at [Fly.io](https://fly.io/dist-sys/).

This repository tries to implement the same using Rust language.
Documentation for the repository can be found in [GitHub pages](https://sak96.github.io/gossip_glomers/).

# Usage

1. List challenges.
  ```bash
  cargo xtask list
  ```

2. Run challenges.
  ```bash
  cargo xtask run --release "$CHALLANGE"
  ```

3. Serve Results.
  ```bash
  cargo xtask serve
  ```

If maelstrom binary is not in `$PATH` variable then for run or serve commands:
  - add flag `-m ./maelstrom` or
  - add environment variable `MAELSTROM_BIN="./maelstrom"`
