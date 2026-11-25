# Kwaliteit

Deze commit wijzigt alleen documentatie; de meting hieronder geldt voor codecommit 4a90b3f.

Verslag van 2026-11-20, aangevuld op 2026-11-24. Toolchains: stable rustc 1.99.0 (b940084d7 2026-09-28) met cargo 1.99.0 (5f94df478 2026-08-27), en rustc 1.85.1 (4eb161250 2025-03-15) met cargo 1.85.1 (d73d2caf9 2024-12-31). Host: Linux x86_64. Tak `poets-cdn`. De controles hieronder draaiden op de code van commit `4a90b3f0b5e4edfc291a05aad9d1d69a21659ddb` (`4a90b3f`): `cargo fmt --check`, beide clippy-runs (stable en 1.85), `cargo test` (debug en release, stable en 1.85, 229 geslaagd) en de pakketlijst. De andere rijen van de Commando's-tabel (clippy en check voor andere doelplatforms, musl-build, glibc-controle, deny, audit, package en de test in het uitgepakte pakket) draaiden op 2026-11-20 op `591915f` en zijn niet opnieuw uitgevoerd op `4a90b3f`: niet uitgevoerd. Commit `4a90b3f` wijzigt `.github/workflows/release.yml` en voegt `tests/archive_links.rs` toe; de commits erna wijzigen alleen documentatie. Versie 1.0.0 is niet gepubliceerd en heeft geen tag.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport. De Engelse [QUALITY.md](QUALITY.md) is de volledige bron; hier staan de samenvatting, de commando's, de uitkomsten, wat niet is uitgevoerd en de beweringen van deze sessie. De lange gevalsgewijze tabellen (stdout-sonde, gesloten en volle stdout, gebruiksfouten met escapes, de volledige tabel beweringen naar test) staan alleen in het Engels in QUALITY.md.

## Bron van het logformaat

De invoervorm die `tests/documented_shape.rs` test (33 velden, `-` voor geen waarde, procent-gecodeerde paden, een kop zonder rijen) is het standaard (legacy) toegangslog van CloudFront zoals beschreven op https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/standard-logging-legacy-s3.html .

## Suite

Het aantal is het aantal `#[test]`-regels onder `src/` en `tests/`. Doc-tests tellen niet mee: de README heeft geen Rust-voorbeelden. `cargo test --locked` gaf 229 geslaagde tests: 123 in de bibliotheek, 0 in het programma, 1 in `tests/archive_links.rs`, 6 in `tests/claims.rs`, 72 in `tests/cli.rs`, 5 in `tests/docs_clean.rs`, 4 in `tests/documented_shape.rs`, 2 in `tests/examples_golden.rs`, 1 in `tests/handbook_help.rs`, 4 in `tests/json_schema.rs`, 9 in `tests/output.rs`, 1 in `tests/sigint.rs`, 1 in `tests/version.rs`, 0 doc-tests. 0 mislukt, 0 genegeerd. Debug, release, Rust 1.85 (debug) en Rust 1.85 (release) geven dezelfde aantallen.

## Commando's

Elk commando draaide apart in de projectmap, met twee compile-jobs (`-j 2`) en `--test-threads 2` voor tests, met `--locked --offline` waar genoemd.

| Commando | Exit |
| --- | ---: |
| `cargo fmt --check` | 0 |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | 0 |
| `cargo +1.85 clippy --locked --offline --all-targets -- -D warnings` | 0 |
| `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings` | 0 |
| `cargo clippy --locked --offline --all-targets --target x86_64-apple-darwin -- -D warnings` | 0 |
| `cargo check --locked --offline --tests --target x86_64-pc-windows-gnu` | 0 |
| `cargo check --locked --offline --tests --target x86_64-apple-darwin` | 0 |
| `cargo build --locked --offline --release --target x86_64-unknown-linux-musl` | 0 |
| `cargo test --locked --offline --no-fail-fast -- --test-threads 2` | 0 |
| `cargo test --locked --offline --no-fail-fast --release -- --test-threads 2` | 0 |
| `cargo +1.85 test --locked --offline --no-fail-fast -- --test-threads 2` | 0 |
| `cargo +1.85 test --locked --offline --no-fail-fast --release -- --test-threads 2` | 0 |
| `cargo build --locked --offline --release`, then `bash scripts/check-glibc.sh target/release/cdn-egress-bill 2.34` (highest symbol GLIBC_2.34) | 0 |
| `cargo deny --offline check licenses` | 0 |
| `cargo audit --no-fetch` (29 crates, 1527 advisories) | 0 |
| `cargo package --locked --offline --allow-dirty --no-verify --list` (52 paths) | 0 |
| `cargo package --locked --offline --allow-dirty --no-verify` | 0 |
| `cargo test --offline --no-fail-fast -- --test-threads 2` in the unpacked `.crate` made by `cargo package` (see below) | 0 |

The internal-name check is `tests/docs_clean.rs`, part of the suite above.

De twee `--target`-controles (en de twee clippy-runs voor die doelen) compileren alleen voor Windows en macOS; daar draaide niets. Het uitgepakte pakket (`cargo package`, uitgepakt in een lege map) draaide op `591915f` 228 tests groen; op `4a90b3f`, dat `tests/archive_links.rs` toevoegt, is dat niet opnieuw uitgevoerd.

## Testresultaat per doel

Debug (`cargo test --locked --offline --no-fail-fast -- --test-threads 2`):

| Doel | Geslaagd | Mislukt | Genegeerd |
| --- | ---: | ---: | ---: |
| unittests `src/lib.rs` | 123 | 0 | 0 |
| unittests `src/main.rs` | 0 | 0 | 0 |
| `tests/archive_links.rs` | 1 | 0 | 0 |
| `tests/claims.rs` | 6 | 0 | 0 |
| `tests/cli.rs` | 72 | 0 | 0 |
| `tests/docs_clean.rs` | 5 | 0 | 0 |
| `tests/documented_shape.rs` | 4 | 0 | 0 |
| `tests/examples_golden.rs` | 2 | 0 | 0 |
| `tests/handbook_help.rs` | 1 | 0 | 0 |
| `tests/json_schema.rs` | 4 | 0 | 0 |
| `tests/output.rs` | 9 | 0 | 0 |
| `tests/sigint.rs` | 1 | 0 | 0 |
| `tests/version.rs` | 1 | 0 | 0 |
| doc-tests | 0 | 0 | 0 |
| totaal | 229 | 0 | 0 |

Release (`cargo test --locked --offline --no-fail-fast --release -- --test-threads 2`):

| Doel | Geslaagd | Mislukt | Genegeerd |
| --- | ---: | ---: | ---: |
| unittests `src/lib.rs` | 123 | 0 | 0 |
| unittests `src/main.rs` | 0 | 0 | 0 |
| `tests/archive_links.rs` | 1 | 0 | 0 |
| `tests/claims.rs` | 6 | 0 | 0 |
| `tests/cli.rs` | 72 | 0 | 0 |
| `tests/docs_clean.rs` | 5 | 0 | 0 |
| `tests/documented_shape.rs` | 4 | 0 | 0 |
| `tests/examples_golden.rs` | 2 | 0 | 0 |
| `tests/handbook_help.rs` | 1 | 0 | 0 |
| `tests/json_schema.rs` | 4 | 0 | 0 |
| `tests/output.rs` | 9 | 0 | 0 |
| `tests/sigint.rs` | 1 | 0 | 0 |
| `tests/version.rs` | 1 | 0 | 0 |
| doc-tests | 0 | 0 | 0 |
| totaal | 229 | 0 | 0 |

Rust 1.85 debug (`cargo +1.85 test ...`) en Rust 1.85 release (`cargo +1.85 test ... --release`) gaven dezelfde aantallen per doel en totaal 229.

## Niet uitgevoerd

- Een fuzzcampagne en een mutatierun.
- GitHub Actions (het CI-bestand en `.github/workflows/release.yml`). De workflows zijn niet gestart (GitHub Actions kan voor dit account op dit moment niet draaien vanwege een betaalprobleem); van geen van beide bestanden is een run gezien. De release-workflow is alleen als YAML ingelezen, dus de gewijzigde `cp`-regels en de nieuwe linkcontrole in de rooktest zijn niet op GitHub gedraaid: niet uitgevoerd.
- `cargo about generate about.hbs`: `cargo-about` is niet geïnstalleerd op deze host, dus `THIRD-PARTY-LICENSES.html` is niet gemaakt en `about.toml` is niet gedraaid. De lijst toegestane licenties is die van `deny.toml`, en `cargo deny check licenses` slaagde.
- Tests op de musl-build, Clippy 1.85 voor de Windows- en macOS-doelen, en Windows en macOS zelf: de code compileert voor `x86_64-pc-windows-gnu` en `x86_64-apple-darwin`, maar daar draaide geen test.
- Echte CloudFront-logs (alleen synthetische) en live AWS-prijzen (de tabel in `data/prices.txt`, gecontroleerd op 2026-11-09).
- `git push`, `git tag` en `cargo publish`.

## Rood vóór groen (sessie van 2026-11-20)

Elke test draaide vóór de wijziging en faalde daar.

| Test | Vóór de wijziging |
| --- | --- |
| `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` | faalde: `ExitStatus(unix_wait_status(2))`, code `None` in plaats van `Some(130)` |
| `tests/output.rs` | 6 van 9 faalden: `error: unexpected argument '--output' found` |
| `tests/json_schema.rs` | 3 van 4 faalden: de JSON begon met `"estimate": true` en had geen `schema_version` |
| `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` | faalde: links `"cdn-egress-bill 1.0.0\n"`, rechts `"cdn-egress-bill 1.0.0 (e5c1ebb)\n"` |
| `tests/examples_golden.rs` | 2 van 2 faalden: `examples/access.report.md` en `examples/README.md` bestonden niet |

## Rood vóór groen (wijziging van 2026-11-24)

Het releasearchief was niet zelfstandig: de `Package`-stap van `.github/workflows/release.yml` kopieerde `QUALITY.nl.md` (waar `README.nl.md` naar linkt), `SECURITY.md` en `data/` (beide README's noemen ze) niet. De test `every_relative_link_in_the_packaged_documents_points_into_the_archive` in `tests/archive_links.rs` leest de `cp`-regels van die stap en volgt elke relatieve link in de gekopieerde documenten, plus de `.md`-bestanden op het hoogste niveau en de `data/`-bestanden die de README's noemen. Op de workflow van commit 87da8e3 (nieuw testbestand, oude `release.yml`) faalde hij met:

```
---- every_relative_link_in_the_packaged_documents_points_into_the_archive stdout ----

thread 'every_relative_link_in_the_packaged_documents_points_into_the_archive' (…) panicked at tests/archive_links.rs:135:5:
not in the release archive: [
    "README.md -> SECURITY.md",
    "README.md -> data/edges.txt",
    "README.md -> data/prices.txt",
    "README.nl.md -> QUALITY.nl.md",
    "README.nl.md -> SECURITY.md",
    "README.nl.md -> data/edges.txt",
    "README.nl.md -> data/prices.txt",
]

failures:
    every_relative_link_in_the_packaged_documents_points_into_the_archive

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Na de wijziging van `release.yml` slaagt hij. In de commit die hem toevoegt faalt `quality_and_readme_share_suite_count` in `tests/claims.rs` (229 geteld, 228 vermeld); de volgende commit, die alleen documentatie wijzigt, vermeldt 229. De rooktest in de workflow controleert dezelfde links in het uitgepakte archief; die is met dezelfde shellregels met de hand gedraaid op een kopie met de nieuwe `cp`-regels (exit 0) en zonder `SECURITY.md` (exit 1). GitHub heeft hem niet gedraaid: niet uitgevoerd.

## Bewering naar test (deze sessie)

| # | Bewering | Test | Status |
| --- | --- | --- | --- |
| R64 | SIGINT beëindigt de run met exit 130 en de regel `cdn-egress-bill: interrupted by SIGINT`; het rapport wordt pas geschreven als het klaar is, maar een signaal tijdens het schrijven kan een deel op stdout achterlaten (de test laat het schrijven op een volle pipe blokkeren en controleert exit en stderr, niet de bytes op stdout) | `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` | getest (een volle pipe houdt het rapport vast; een signaal tijdens de berekening gebruikt dezelfde handler en is niet getimed) |
| R65 | `--output` schrijft het hele rapport via een tijdelijk bestand en één hernoeming, vervangt een bestand of symbolische link zonder te volgen, weigert een map en geeft exit 2 met een melding bij een schrijffout | `output_file_holds_exactly_the_report_that_stdout_would_have`, `an_existing_file_is_replaced_by_the_whole_new_report`, `a_missing_parent_directory_is_exit_2_and_is_not_created`, `a_directory_as_output_is_refused_and_kept`, `a_failing_run_leaves_an_existing_output_file_alone`, `a_symlink_as_output_is_replaced_and_its_target_is_not_written`, `a_name_with_control_characters_is_escaped_in_the_error`, `strict_still_exits_3_and_writes_the_file`, `output_without_a_value_is_a_usage_error` | getest; een SIGINT die tijdens het hernoemen wordt vastgehouden is niet getest (gat) |
| R66 | `--version` is de crateversie, met de korte commit in een debug-build uit een git-checkout | `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` | getest |
| R67 | Elk commando van `examples/README.md` geeft de bewaarde tekst-, Markdown- en JSON-uitvoer | `each_example_command_gives_its_expected_output`, `the_examples_readme_names_every_command_and_file` | getest |
| R68 | Het releasearchief bevat elk bestand waar de documenten naar linken of dat ze noemen: `QUALITY.nl.md`, `SECURITY.md`, `data/prices.txt` en `data/edges.txt` | `every_relative_link_in_the_packaged_documents_points_into_the_archive` | getest op de `cp`-regels; het archief zelf is niet gebouwd (niet uitgevoerd) |
