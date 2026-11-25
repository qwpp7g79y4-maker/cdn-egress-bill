# Contributing

Thanks for your interest in cdn-egress-bill.

## Building and testing

```sh
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

These checks must pass before a change is merged. Test fixtures must be synthetic, or anonymised copies of a public post. Never add a real CloudFront log, a real distribution id, a real client address or a real AWS bill.

## Commit messages

Commit messages are short English sentences that say what changed, for example `Escape controls in values shown on the report`.

## Contact

PEPAI, pep_ai@icloud.com
