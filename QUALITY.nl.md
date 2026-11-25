# Kwaliteit

Verslag van 2026-11-25. De controles hieronder draaiden op commit `c9f1f66cedb97bd7a037a96c97febfee8168d4fc` (`c9f1f66`), de top van de tak `stdout-probe`, in een schone worktree. De commit die deze pagina vastlegt wijzigt alleen documentatie (dit bestand en `QUALITY.md`).

Toolchain: stable rustc 1.99.0 (b940084d7 2026-09-28) met cargo 1.99.0 (5f94df478 2026-08-27). Host: Linux x86_64. Versie 1.0.0 is niet gepubliceerd en heeft geen tag.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport. De Engelse [QUALITY.md](QUALITY.md) is de bron.

## Bron van de prijzen

De tool bouwt de AWS CloudFront pay-as-you-go-lijstprijzen na (Data Transfer Out to Internet en HTTP(S)-verzoeken). De documentatie die hij nabouwt staat op https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ . De momentopname in `data/prices.txt` is gecontroleerd op 2026-11-09. Deze meting haalde die pagina niet opnieuw op.

## Commando's

Elk commando draaide apart in de projectmap, met de getoonde vlaggen en geen andere.

| Commando | Exit |
| --- | ---: |
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | 0 |
| `cargo test --locked` | 0 |
| `cargo test --release --locked` | 0 |

## Rust 1.85

Toegevoegd op 2026-11-25. De vier commando's hieronder draaiden met `cargo +1.85` (cargo 1.85.1 (d73d2caf9 2024-12-31)), op tak `stdout-probe` bij commit `c9f1f66`, zonder codewijziging sinds de commit bovenaan. Elk draaide apart, met de getoonde vlaggen en geen andere.

| Commando | Exit |
| --- | ---: |
| `cargo +1.85 fmt --check` | 0 |
| `cargo +1.85 clippy --all-targets --locked -- -D warnings` | 0 |
| `cargo +1.85 test --locked` | 0 |
| `cargo +1.85 test --release --locked` | 0 |

`cargo +1.85 test --locked` gaf 229 geslaagde tests, met dezelfde aantallen per bestand als hieronder. 0 mislukt, 0 genegeerd. `cargo +1.85 test --release --locked` gaf 229 geslaagde tests, 0 mislukt, 0 genegeerd (2026-11-25).

## Suite

`cargo test --locked` gaf 229 geslaagde tests: 123 in de bibliotheek, 0 in het programma, 1 in `tests/archive_links.rs`, 6 in `tests/claims.rs`, 72 in `tests/cli.rs`, 5 in `tests/docs_clean.rs`, 4 in `tests/documented_shape.rs`, 2 in `tests/examples_golden.rs`, 1 in `tests/handbook_help.rs`, 4 in `tests/json_schema.rs`, 9 in `tests/output.rs`, 1 in `tests/sigint.rs`, 1 in `tests/version.rs`, 0 doc-tests. 0 mislukt, 0 genegeerd. `cargo test --release --locked` gaf dezelfde aantallen.

## Niet uitgevoerd

Deze meting deed het volgende niet; elk is not run (niet uitgevoerd):

- Clippy en `cargo check` voor `x86_64-pc-windows-gnu` en `x86_64-apple-darwin`: not run.
- De musl-releasebouw en `scripts/check-glibc.sh`: not run.
- `cargo deny`, `cargo audit`: not run.
- `cargo package` (lijst, bouw en de tests in het uitgepakte pakket): not run.
- `--offline`, `-j 2` en `--test-threads 2` zijn niet gebruikt; de commando's draaiden met de standaardwaarden van de host.
- De tabellen van eerdere verslagen (stdoutsonde, beweringen per uitspraak, limieten, looptijden) staan hier niet en zijn niet opnieuw gemeten: not run.
- Geen mutatierun, geen fuzzing: not run.
