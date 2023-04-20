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

If maelstrom binary is not in `$PATH` variable then, add flag `-m $MALESTROM_LOCATION` run or serve commands.

# To-do list

- Kafka challenge
  - [ ] single node
  - [ ] multi node
  - [ ] efficient
- Totally-Available Transactions
  - [ ] single node
  - [ ] read uncommitted
  - [ ] read committed

# Testing

Naive test cases are in current package.
These can be run as follow:

```bash
cargo test
```

The integration test cases are used only for checking the request/response parsing.
Some integration test cases are ignored due to race conditions.

To run all challenge as test cases. 
```bash
cargo test -p xtask
```
