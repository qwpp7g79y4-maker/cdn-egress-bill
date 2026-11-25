# Examples

A small invented CloudFront standard log: no real distribution, client or bill. Run every command from the repository root. The reports are estimates from list prices, not an AWS invoice. A price-age warning may appear on stderr; it does not change the output files.

## Text report

```sh
cdn-egress-bill examples/access.log
```

Expected output: `examples/access.report.txt`. Expected exit code: 0.

## Markdown report

```sh
cdn-egress-bill --md examples/access.log
```

Expected output: `examples/access.report.md`.

## JSON report

```sh
cdn-egress-bill --format json examples/access.log
```

Expected output: `examples/access.report.json`. It starts with `"schema_version": 1`.

The 50 bytes of `/secret` come from an edge that is not in the map, so that region is `onbekend` and is not priced. With `--strict` the same command exits 3.

Nothing here writes into this folder: use `--output` with a path of your own when you want a file.
