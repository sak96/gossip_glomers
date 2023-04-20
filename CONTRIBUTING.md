# Setup

Download maelstrom from [here](https://github.com/jepsen-io/maelstrom/releases).
Add it binary to `$PATH` or pass it using `-m` flag to xtask.

# Challenge

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
