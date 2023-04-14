#  Gossip Glomers

![Lint and Deploy docs](https://github.com/sak96/gossip_glomers/actions/workflows/main.yml/badge.svg)
![License](https://img.shields.io/badge/License-Apache_2.0-yellowgreen.svg)
![Top Language](https://img.shields.io/github/languages/top/sak96/gossip_glomers)

`Gossip Glomers` is series of distributed systems challenges.
This project tries to implement the same using Rust language.
More details of these challenges can be found at [Fly.io](https://fly.io/dist-sys/).

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

If maelstrom binary is not in `$PATH` variable then, add flag `-m $MALESTROM_LOCATION` run or serve commands.
