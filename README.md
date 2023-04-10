#  Gossip Glomers

A series of distributed systems challenges brought to you by [Fly.io](https://fly.io/dist-sys/).
This project tries to implement the same using Rust language.

# Usage

1. List challenges.
  ```bash
  cargo xtask list
  ```

2. Run challenges.
  ```bash
  cargo xtask run "$CHALLANGE"
  ```

3. Serve Results.
  ```bash
  cargo xtask serve
  ```

If maelstrom binary is not in `$PATH` variable then, add flag `-m $MALESTROM_LOCATION` run or serve commands.
