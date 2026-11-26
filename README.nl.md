# cdn-egress-bill

[![crates.io](https://img.shields.io/crates/v/cdn-egress-bill.svg)](https://crates.io/crates/cdn-egress-bill) [![downloads](https://img.shields.io/crates/d/cdn-egress-bill.svg)](https://crates.io/crates/cdn-egress-bill) [![CI](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/actions/workflows/ci.yml/badge.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/actions/workflows/ci.yml) [![PEPAI checked](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fqwpp7g79y4-maker%2Fcdn-egress-bill%2Fmain%2F.github%2Fbadge.json)](QUALITY.md) [![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/blob/main/Cargo.toml) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill#license)

Schat CloudFront-egressiekosten uit standaardlogbestanden. Leest alleen lokale bestanden: er is geen AWS SDK en geen netwerkclient.

De tests en de gemeten getallen staan in `QUALITY.md`.

## Waarom

Een CloudFront-standaardlog noemt al per verzoek de verzonden bytes, de edge-locatie en het pad. cdn-egress-bill prijst die logs op uw eigen machine, per prijsregio en per pad, met de gratis terabyte en de gratis verzoeken toegepast. Het programma roept AWS niet aan en heeft geen account nodig.

De beschrijvingen in de `--help`-tekst en het rapport zijn Nederlands en staan in dit bestand precies zoals het programma ze print. De koppen en vaste woorden van `--help` (`Usage:`, `Options:`, `Print help`, `[default: text]`) komen van de argumentparser en zijn Engels. De meeste foutmeldingen zijn Nederlands. Enkele zijn Engels: `file is larger than 268435456 bytes`, `gzip: truncated gzip header`, `gzip: trailing data is not a gzip member`, `stream did not contain valid UTF-8` en `overflow`. Een gebruiksfout komt in het Engels van de argumentparser, en de tekst na een bestandsnaam of na `stdout:` komt van het besturingssysteem.

## Installatie

Versie 1.0.0. Alleen op Linux (x86_64) gemeten; zie [QUALITY.nl.md](QUALITY.nl.md) voor wat wel en niet is uitgevoerd. Van crates.io, zodra 1.0.0 daar staat:

```sh
cargo install cdn-egress-bill
```

Of bouw uit de broncode, met Rust 1.85 of nieuwer, vanuit een kloon van deze repository:

```sh
cargo install --locked --path .
cdn-egress-bill --help
```

Dit crate is niet gepubliceerd en heeft geen git-tag. Installeer tot dan vanuit deze checkout. De tests die `cargo test --locked` draait, zijn geteld in [QUALITY.md](QUALITY.md).


## Snelstart

Een klein log met de kolommen `date`, `x-edge-location`, `sc-bytes`, `cs-uri-stem` en `cs-protocol`, en het rapport ervoor (de opties staan onder Opties en gebruik):

```sh
printf '#Version: 1.0\n#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-03-01\tAMS50-C1\t1000\t/video.mp4\thttps\n2026-03-01\tIAD89-P1\t2000\t/img.png\thttp\n2026-03-02\tZZZ1-C1\t50\t/secret\thttps\n' > access.log
cdn-egress-bill access.log
```

Hetzelfde log en rapport staan in `examples/access.log` en `examples/access.report.txt`. Het rapport is door tabs gescheiden. De 50 bytes van `/secret` komen van een edge die niet in de kaart staat, dus die regio is `onbekend` en wordt niet berekend (`-`):

```
bron: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/
gecontroleerd: 2026-11-09
bytes_per_gb: 1073741824
gratis_data_bytes: 1099511627776
gratis_verzoeken: 10000000
Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)
geen flat-rate, geen korting, geen Savings Bundle
regio = prijsregio van de edge-locatie (x-edge-location), niet het land van de kijker

maand: 2026-03
per regio
regio	edges	bytes	http	https	overig	usd	berekend
US/Mexico/Canada	IAD	2000	1	0	0	0.000000	ja
Europe/Israel/Türkiye	AMS	1000	0	1	0	0.000000	ja
onbekend	ZZZ	50	0	1	0	-	nee
per pad
pad	bytes	verzoeken	usd	onberekend_bytes	onberekend_verzoeken
/img.png	2000	1	0.000000	0	0
/secret	50	1	0.000000	50	1
/video.mp4	1000	1	0.000000	0	0
estimated_usd_month=0.000000
gratis_data_toegepast: 3000
gratis_verzoeken_toegepast: 2
onberekend_bytes: 50
onberekend_verzoeken: 1
```

## Aan de slag in 5 minuten

1. Bouw of installeer de tool (zie Installatie): `cargo install --locked --path .`.
2. Zet een CloudFront-standaardlog (tekst of gzip) in een map, bijvoorbeeld `access.log`.
3. Draai `cdn-egress-bill access.log`: de uitvoer is een schatting uit lijstprijzen, geen AWS-factuur.
4. Rekent u liever per maand? Voeg `--month 2026-03` toe. Wilt u een bestand? Voeg `--output rapport.txt` toe. Wilt u Markdown of JSON? Gebruik `--md` of `--format json`.
5. Een fout geeft exit 2 en een melding op stderr; zie Afsluitcodes.

## Opties en gebruik

```
cdn-egress-bill [--prices FILE] [--edges FILE] [--format text|json] [--month YYYY-MM] [--as-of YYYY-MM-DD] [--md] [--output FILE] [--quiet | --verbose] [--strict] LOG [LOG ...]
```

`LOG` is een CloudFront-standaardlog, als tekst of gzip. Zonder `--prices` en `--edges` gelden de ingebouwde bestanden `data/prices.txt` en `data/edges.txt`. `--format` is standaard `text`. `--month` houdt één maand over (UTC, `YYYY-MM`). `--as-of YYYY-MM-DD` (standaard: vandaag, UTC) is de datum waartegen de ouderdom van de prijzen wordt gemeten: ligt de `checked_on`-datum van het prijsbestand (ingebouwd of `--prices`) meer dan 180 dagen daarvoor, dan gaat er één waarschuwing met die datum naar stderr en blijft de exitcode 0. `--output FILE` schrijft het rapport naar `FILE` in plaats van naar stdout: het gaat eerst naar een tijdelijk bestand in dezelfde map en wordt dan over `FILE` heen hernoemd, zodat het bestand het oude of het hele nieuwe is, nooit een half rapport. Een bestaand bestand wordt vervangen, een symbolische link op `FILE` wordt vervangen en niet gevolgd, en een map wordt geweigerd. Een ontbrekende bovenliggende map (die wordt niet aangemaakt), een map als doel of een schrijffout is exit 2 met een melding. `--strict` (exit 3) schrijft het bestand ook. `--quiet` print alleen het rapport en fouten (geen waarschuwing); `--verbose` voegt voortgangsregels op stderr toe (prijsdatum, elk log, aantallen records en maanden) en laat stdout zoals het is. Samen zijn ze een gebruiksfout, exit 2. `--version` print de crateversie; een debug-build uit een git-checkout van dit crate voegt de korte commit toe, bijvoorbeeld `cdn-egress-bill 1.0.0 (a1b2c3d)`, en een release-build of uitgepakt pakket print alleen de versie.

## Voorbeeld

Het rapport is een schatting uit lijstprijzen, geen AWS-factuur: de regel `Estimate from list prices, ...` staat één keer, elke maand eindigt zijn geld met `estimated_usd_month=`, en de JSON begint met `"schema_version": 1` (de versie van de JSON-indeling; die verandert als een sleutel een andere naam, wegvalt of een andere betekenis krijgt) en heeft `"estimate": true` bovenaan en `estimated_usd_month_micros` per maand. `--md` print hetzelfde rapport als Markdown (niet te combineren met `--format`, exit 2): dezelfde bedragen als het tekstrapport in tabellen, met `|` en `\` in een cel geschreven als `\|` en `\\`, en stuurtekens als `\u000a`. Hetzelfde log geeft:

```
# CDN egress estimate

Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)

- bron: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/
- gecontroleerd: 2026-11-09
- bytes_per_gb: 1073741824
- gratis_data_bytes: 1099511627776
- gratis_verzoeken: 10000000

geen flat-rate, geen korting, geen Savings Bundle

regio = prijsregio van de edge-locatie (x-edge-location), niet het land van de kijker

## 2026-03

### per regio

| regio | edges | bytes | http | https | overig | usd | berekend |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US/Mexico/Canada | IAD | 2000 | 1 | 0 | 0 | 0.000000 | ja |
| Europe/Israel/Türkiye | AMS | 1000 | 0 | 1 | 0 | 0.000000 | ja |
| onbekend | ZZZ | 50 | 0 | 1 | 0 | - | nee |

### per pad

| pad | bytes | verzoeken | usd | onberekend_bytes | onberekend_verzoeken |
| --- | --- | --- | --- | --- | --- |
| /img.png | 2000 | 1 | 0.000000 | 0 | 0 |
| /secret | 50 | 1 | 0.000000 | 50 | 1 |
| /video.mp4 | 1000 | 1 | 0.000000 | 0 | 0 |

estimated_usd_month=0.000000

- gratis_data_toegepast: 3000
- gratis_verzoeken_toegepast: 2
- onberekend_bytes: 50
- onberekend_verzoeken: 1
```

Tekst toont USD met zes decimalen. Een gratis bedrag is `0.000000`. Een regio zonder prijs toont `-`. JSON schrijft micro-USD als geheel getal (1 USD = 1000000).

## Hoe het telt

Een log is gzip als de bestandsnaam op `.gz` eindigt (elke hoofdletterstand) of de eerste twee bytes `1f 8b` zijn; anders is het UTF-8-tekst. Een gzip-bestand mag uit meerdere members bestaan: elk member wordt gelezen, elk member mag met een BOM beginnen, en nulbytes na een member worden genegeerd. Andere data na een member is een fout. Een bestand met meer dan 500000 members is ook een fout (`gzip: more than 500000 members`, exit 2). Regels eindigen op LF, CRLF of een losse CR. De `#Fields:`-regel bepaalt de kolommen. `date`, `x-edge-location`, `sc-bytes`, `cs-uri-stem` en `cs-protocol` zijn verplicht; de volgorde maakt niet uit, en een herhaalde veldnaam is een fout. Een regel met een ander aantal velden, een ongeldige datum of een `sc-bytes` dat geen getal is, is een fout met bestand en regelnummer.

De tarieven en de controledatum staan in `data/prices.txt`. Dat is de enige prijstabel. Bron: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ . Gecontroleerd op 2026-11-09.

De kosten komen uit `sc-bytes`. Elke regel telt, ook HTTP 404. Het pad is `cs-uri-stem`, ongewijzigd. `sc-content-len` telt niet mee.

De prijsregio komt uit `x-edge-location`. De eerste drie letters zijn de IATA-code van de edge. De regio is de prijsregio van de edge-locatie (`x-edge-location`), niet het land van de kijker. Een onbekende code is `onbekend` en wordt niet geraden. China heeft een aparte prijslijst, staat niet in de kaart en blijft `onbekend`.

Eén gigabyte is 1073741824 bytes. De eerste 1 TB per maand is gratis voor alle regio's samen. De eerste 10000000 HTTP- en HTTPS-verzoeken per maand zijn samen gratis. Elke maand heeft zijn eigen gratis ruimte. Bytes en verzoeken op een onbekende edge gebruiken die ruimte niet op. Een ander protocol dan HTTP of HTTPS gebruikt de gratis verzoeken niet op. Bij een gelijk verzoektarief gaat de gratis ruimte eerst van HTTPS af, daarna van HTTP.

1 GB boven die gratis terabyte in US/Mexico/Canada kost 0,085 USD. Verdere trappen staan alleen in `data/prices.txt`.

Hetzelfde pad twee keer als argument geeft telt twee keer.

Geen flat-rate, geen korting, geen Savings Bundle.

Na de regiokosten wordt het bedrag naar verhouding over de paden verdeeld: data naar bytes, verzoeken naar aantal. De verdeling hangt niet af van de volgorde van de logregels. Reste-micro's gaan naar de grootste restbreuk, dan het grootste gewicht, dan de padnaam, zodat de paden optellen tot het regiobedrag.

### Beweringen

| Bewering | Test |
| --- | --- |
| 1 GB boven de gratis terabyte in de VS kost 0,085 USD | `one_gb_above_the_free_terabyte_uses_the_us_rate` |
| Een log in de gedocumenteerde standaardlogvorm geeft het exacte bedrag | `the_full_documented_log_costs_exactly_one_gb_above_the_free_terabyte` |
| De schattingsregel, `estimated_usd_month=` en `"estimate": true` | `text_labels_the_monthly_cost_as_an_estimate` |
| Elk JSON-rapport begint met `"schema_version": 1`; een fout geeft geen JSON | `every_json_report_has_schema_version_1_first`, `schema_version_is_also_there_with_strict_quiet_verbose_and_as_of`, `schema_version_is_also_in_a_json_file_from_output`, `an_error_with_json_selected_prints_no_json` |
| Prijzen ouder dan 180 dagen waarschuwen op stderr, exit 0 | `prices_older_than_180_days_warn_on_stderr_and_exit_0` |
| SIGINT tijdens het schrijven van het rapport geeft exit 130 met de onderbrekingsregel | `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` |
| `--output` schrijft precies het rapport dat stdout zou geven, via een tijdelijk bestand en hernoemen | `output_file_holds_exactly_the_report_that_stdout_would_have`, `an_existing_file_is_replaced_by_the_whole_new_report`, `strict_still_exits_3_and_writes_the_file` |
| `--output`-fouten (ontbrekende map, map als doel) zijn exit 2 en laten het oude bestand staan | `a_missing_parent_directory_is_exit_2_and_is_not_created`, `a_directory_as_output_is_refused_and_kept`, `a_failing_run_leaves_an_existing_output_file_alone`, `a_name_with_control_characters_is_escaped_in_the_error` |
| `--output` volgt geen symbolische link | `a_symlink_as_output_is_replaced_and_its_target_is_not_written` |
| `--version` is de crateversie, plus de commit in een debug-checkout | `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` |
| Elk commando in `examples/README.md` geeft de bewaarde tekst-, Markdown- en JSON-uitvoer | `each_example_command_gives_its_expected_output`, `the_examples_readme_names_every_command_and_file` |
| 1 TB in één regio is gratis | `one_tebibyte_in_one_region_is_free` |
| De gratis terabyte gaat eerst van de duurdere regio af | `free_terabyte_comes_off_the_dearer_region` |
| HTTPS boven 10000000 verzoeken volgt de tabel | `https_above_the_free_requests_uses_the_table` |
| De testlog heeft twee regio's en twee paden | `fixture_has_two_regions_and_two_paths` |
| Een onbekende edge heet onbekend | `fixture_labels_unknown_edge_onbekend` |
| Luchthavens in China zijn onbekend | `china_airports_are_onbekend` |
| Het rapport zegt dat de edge niet het land van de kijker is | `text_report_says_edge_is_not_the_viewer` |
| Onbekend wordt niet berekend | `text_report_marks_onbekend_unpriced` |
| Reste-micro's gaan naar de grootste rest | `remainder_micros_go_to_the_larger_remainder` |
| Elke maand heeft zijn eigen gratis terabyte | `months_each_get_their_own_free_terabyte` |
| Strict geeft exit 3 en print het rapport toch | `strict_exits_3_and_still_prints_onbekend` |
| De paden tellen op tot het maandtotaal | `path_costs_sum_to_the_month_total` |
| Bij gelijke rest gaat de micro naar het grootste gewicht | `equal_remainder_goes_to_the_larger_weight` |
| Bij gelijk gewicht gaat de micro naar de eerdere padnaam | `equal_weight_remainder_goes_to_the_earlier_path` |
| Bij gelijk verzoektarief gaat de gratis ruimte eerst van HTTPS af | `equal_request_rates_waive_https_before_http` |
| Het manifest bevat geen AWS SDK | `manifest_has_no_aws_sdk` |
| De snelstartlog en het rapport zijn de bestanden in `examples/` en wat het programma print | `quick_start_log_and_report_match_the_examples_and_the_binary` |
| Een gzip-log wordt herkend aan de eerste bytes of aan een `.gz`-naam in elke hoofdletterstand | `gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name` |
| Een gzip-member mag midden in een UTF-8-teken eindigen | `gzip_members_may_split_a_utf8_character` |
| De volgorde van de logregels verandert het rapport niet | `the_order_of_the_log_lines_does_not_change_the_report` |
| Datums en `--month` lopen van 1970 tot 2100 | `dates_and_months_are_accepted_from_1970_to_2100_only` |
| `sc-bytes` is een 64-bit getal zonder teken; een som die niet past is een fout | `sc_bytes_is_an_unsigned_64_bit_number_and_a_sum_that_does_not_fit_is_an_error` |
| Een lege log of een log met alleen een kop is een rapport zonder verkeer, exit 0 | `an_empty_log_and_a_header_only_log_succeed_even_with_strict` |
| Een `#`-regel met een tab is een gegevensrij | `a_hash_line_that_holds_a_tab_is_a_data_row` |
| Een gebruiksfout met een escape-reeks en een regeleinde blijft op eigen regels | `clap_error_with_an_escape_sequence_and_a_line_break_stays_on_one_line` |
| Een gebruiksfout met een argument dat geen UTF-8 is en een regeleinde bevat blijft op eigen regels | `clap_error_with_a_non_utf8_argument_and_a_line_break_stays_on_one_line` |
| Een gebruiksfout voor een argument dat geen UTF-8 is houdt de eigen regels van clap en toont de bytes als `\xff` | `non_utf8_month_value_keeps_clap_lines_and_non_utf8_option_shows_its_bytes` |
| Een grotere rest wint van een groter gewicht | `a_larger_remainder_beats_a_larger_weight` |
| Schrikkeldagen en maandnummers volgen de kalender | `leap_days_and_month_numbers_follow_the_calendar` |
| Eén leeg veld na de laatste tab van een gegevensrij wordt genegeerd, twee zijn een fout | `a_single_trailing_tab_on_a_data_row_is_tolerated` |
| Een bestand van precies 268435456 bytes wordt gelezen, één byte meer wordt geweigerd | `a_file_of_exactly_the_size_limit_is_read_and_one_byte_more_is_refused` |
| Een tekstbestand met de naam `.GZ` wordt geweigerd als slecht gzip-bestand, dus de naamregel geldt op zichzelf | `a_plain_file_with_an_upper_case_gz_name_is_refused_as_gzip` |
| Een gzip-bestand van 500000 members wordt gelezen, een van 500001 members wordt geweigerd met exit 2, en 200000 lege members, met of zonder BOM of naam in elk, worden binnen 30 seconden gelezen | `a_gzip_file_of_more_members_than_the_maximum_is_refused`, `many_empty_gzip_members_are_read_in_time`, `many_gzip_members_with_a_bom_each_are_read_in_time`, `many_gzip_members_with_every_header_part_are_read_in_time` |
| Members met een naam, commentaar of extra veld geven het rapport van één member, en een beschadigd member na veel lege geeft dezelfde melding als alleen | `gzip_members_with_header_parts_give_the_report_of_one_member`, `a_damaged_member_after_empty_members_is_refused_with_the_same_text` |
| Een `#Fields:`-regel van 200000 namen en een prijsbestand van 100000 regio's met labels worden binnen de tijdgrens van de test gelezen, en een herhaalde naam of regio aan het eind wordt nog gevonden | `a_fields_line_with_many_names_is_read_in_linear_time`, `a_price_file_with_many_regions_and_labels_is_read_in_linear_time` |

## Regels voor invoer en grenzen

Gemeten met het release-programma op Linux; de commando's en uitkomsten staan in [QUALITY.md](QUALITY.md).

- Datums: `date` is `YYYY-MM-DD`, een echte kalenderdag van 1970-01-01 tot 2100-12-31. `--month` neemt dezelfde jaren, 1970-01 tot 2100-12. Al het andere is exit 2 met bestand en regelnummer.
- `sc-bytes` bestaat alleen uit decimale cijfers, 0 tot 18446744073709551615. Een teken, een breuk, een hexadecimaal getal, een leeg veld of een groter getal is exit 2. Een som die niet in de bedragenrekenkunde past, stopt de run met `<log>: overflow`, exit 2, en geen rapport; 300 regels van 18446744073709551615 bytes in één regio doen dat.
- Afronding: elk factureerbaar deel wordt half-omhoog afgerond op één micro-USD (1/1000000 USD). Een deel is één prijstrap van één regio voor data, en één protocol van één regio voor verzoeken. Regiobedragen en het maandtotaal zijn sommen van die micro-USD-bedragen en worden niet opnieuw afgerond.
- Een log zonder gegevensrijen (leeg, of alleen commentaar en `#Fields:`-regels) is geen fout: het rapport zegt `geen verkeer` en de exitcode is 0, ook met `--strict`.
- Een gegevensrij mag op één tab eindigen: één leeg veld na de laatste tab wordt genegeerd, twee zijn een veld te veel.
- Een regel die met `#` begint en geen tab heeft, is commentaar. Een regel die met `#` begint en een tab bevat, is een gegevensrij, omdat een veld als `cs(Referer)` met `#` kan beginnen. Een `#Version: x`-regel met een tab midden in een log is dus een fout met het regelnummer.
- Looptijd van een gzip-log met heel veel members. Elk member zet een decoder op, wat ongeveer 4 microseconden kost, ook voor een leeg member of een member met alleen een BOM. Gemeten met het release-programma: een bestand van 124999, 249999 en 499999 lege members na een kopmember (2,5 MB, 5,0 MB, 10,0 MB) kostte 0,44, 0,89 en 1,84 seconden, en met een BOM in elk member 0,46, 0,93 en 1,82 seconden, met hoogstens 14 MB geheugen. Meer dan 500000 members wordt geweigerd, dus een bestand onder de groottegrens kost hierdoor ongeveer 2 seconden (een bestand van 13421700 members, 268434080 bytes, werd na 2,0 seconden geweigerd; vóór de grens duurde het 53 seconden). Een log van meer dan 500000 aaneengeplakte gzip-bestanden moet worden gesplitst.
- Looptijd. De controles op een herhaalde veldnaam, een herhaalde regio en een label voor een regio kosten tijd naar verhouding tot de bestandsgrootte. Gemeten met het release-programma: een `#Fields:`-regel van 2000000 namen (16,9 MB) kostte 0,4 seconden, een prijsbestand van 400000 regio's met labels (22 MB) kostte 1,7 seconden met een log van één regel. De tijd van een prijsbestand groeit ook met het aantal regio's maal het aantal maanden in het log: 40000 regio's en een log met 1572 maanden (elke maand van 1970 tot 2100) kostte 13 seconden, 80000 regio's 33 seconden. Het ingebouwde prijsbestand heeft 8 regio's.
- Een log wordt volledig in het geheugen gelezen. Een tekstlog nam op het hoogtepunt ongeveer 24 keer zijn grootte in beslag (een log van 50000093 bytes met 1345030 verschillende paden: 1,2 GB, 3,6 seconden). De grens van 268435456 bytes geldt voor het bestand zoals opgeslagen. Ze begrenst niet wat een gzip-bestand uitpakt: een gzip van 4,7 MB met 1 GiB nulbytes werd helemaal uitgepakt, 2,1 GB op het hoogtepunt, voordat het werd geweigerd. Splits zeer grote logs en lees geen gzip-bestanden die u niet vertrouwt op een kleine machine.
- Het prijsbestand (`--prices`) bevat regels van de vorm `source_url https://...`, `checked_on YYYY-MM-DD`, `bytes_per_gb N`, `free_data_bytes N`, `free_requests N`, `label ID TEXT` en `region ID R1 R2 R3 R4 R5 R6 R7 HTTP HTTPS`. `R1` tot `R7` zijn de USD per GB van de zeven datatrappen, `HTTP` en `HTTPS` zijn de USD per 10000 verzoeken, met hoogstens 6 decimalen. Elke sleutel moet één keer voorkomen, `ID` bestaat uit kleine letters, cijfers en `_`, een `label` is optioneel en per regio hoogstens één keer. Regels die met `#` beginnen en lege regels worden genegeerd. Velden worden gescheiden door spaties of tabs, en het bestand mag niet met een BOM beginnen. `data/prices.txt` is het volledige voorbeeld.
- De edge-kaart (`--edges`) bevat per regel één `CODE REGION-ID`, met een IATA-code van drie letters, elke code één keer, en een regio-id dat in het prijsbestand bestaat. `data/edges.txt` is het volledige voorbeeld.
- Gebruiksfouten. Wat u typte wordt teruggetoond met elk stuurteken, elke escape, regeleinde, bidi- en onzichtbaar teken geschreven als `\u000a`, `\u001b` enzovoort, zodat een argument geen regel aan de melding kan toevoegen. Een argument dat geen UTF-8 is, wordt als bytes getoond, bijvoorbeeld `\xff`; als het enige probleem is dat een argument geen UTF-8 is (bijvoorbeeld `--month` met zo'n waarde), wordt de eigen melding van clap geprint, omdat die het argument niet herhaalt. Na een enkel streepje kan zo'n argument met één teken worden gemeld (`unexpected argument '-\'`).
- Systeemeisen. Het release-programma, gebouwd op Linux met glibc, vraagt GLIBC_2.34 of nieuwer. Een build voor `x86_64-unknown-linux-musl` heeft geen glibc-eis en print het snelstartrapport byte voor byte. Er is geen minimumversie van macOS of Windows gemeten: de code is voor beide typegecontroleerd en gelint, en er draaide daar geen test.
- Op een ander platform dan Linux draaien de tests die `/dev/full`, `/proc`, een gesloten bestandsdescriptor of een naam die geen UTF-8 is nodig hebben niet, en een gesloten stdout wordt niet herkend (zie boven). Windows en macOS zijn niet uitgevoerd.

## Afsluitcodes

Exitcode 0 is gelukt. Exitcode 130 is SIGINT (Ctrl-C): het programma print `cdn-egress-bill: interrupted by SIGINT` op stderr, begint pas met schrijven naar stdout als het hele rapport berekend is (een SIGINT tijdens de berekening laat stdout leeg, maar een SIGINT tijdens het schrijven van een rapport dat groter is dan de pipe-buffer kan een deel op stdout achterlaten) en geeft nooit een paniek. Met `--output` wacht een SIGINT tijdens het laatste schrijven op het hernoemen, zodat het bestand heel is, en eindigt dan met 130. Exitcode 2 is een invoer- of gebruiksfout: een ongeldige maand, een ontbrekend log, een onleesbaar bestand, of iets dat geen gewoon bestand is (behalve de eigen stdin-pipe). `--strict` print het rapport en geeft exit 3 als er onberekende bytes of verzoeken zijn. Een gewoon bestand (log, `--prices` of `--edges`) boven 268435456 bytes wordt vóór het lezen geweigerd met exit 2. Een pipe op stdin heeft geen lengte: dezelfde grens geldt door te tellen wat binnenkomt, en boven 268435456 bytes stopt het lezen met dezelfde fout en exit 2. Op Linux is een stdout die bij de start al gesloten was exit 2, terwijl `1<>/dev/null` en `>/dev/null` gewone uitvoer blijven. Op een ander unix-systeem is een gesloten stdout niet te onderscheiden van een leesbare `/dev/null` en telt niet als gesloten.

| Code | Betekenis |
| --- | --- |
| 0 | Gelukt |
| 2 | Invoer- of gebruiksfout, ook `--quiet` met `--verbose`, `--md` met `--format` en een `--output` dat niet te schrijven is |
| 3 | `--strict` en sommige bytes of verzoeken konden niet worden berekend (het rapport wordt toch geprint) |
| 130 | SIGINT (Ctrl-C) |

## Beperkingen

Versie 1.0.0 prijst alleen CloudFront pay-as-you-go-egressie uit standaardlogs. Er is geen AWS SDK en geen netwerkaanroep. De ingebouwde tarieven staan in `data/prices.txt`, gecontroleerd op 2026-11-09. China wordt niet berekend. Er is geen flat-rate, geen korting en geen Savings Bundle. Een onbekende edge wordt niet geraden en zijn bytes en verzoeken blijven onberekend. De regio is de edge, niet de kijker. Hetzelfde log twee keer opgegeven telt twee keer. Een bestand boven 268435456 bytes wordt geweigerd, maar een gzip-bestand is na het uitpakken niet begrensd. De helptekst en de meeste meldingen zijn Nederlands, enkele meldingen zijn Engels. Het rapport is door tabs gescheiden tekst voor een terminal. Een pad dat met `=`, `+`, `-` of `@` begint, wordt ongewijzigd geschreven, dus een spreadsheet die de uitvoer opent kan het als formule lezen; echte CloudFront-paden beginnen met `/`. De getallen zijn een schatting, geen factuur.

## Prijzen en verantwoordelijkheid

De ingebouwde prijzen zijn een momentopname van de AWS CloudFront pay-as-you-go-lijstprijzen, van https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ , gecontroleerd op 2026-11-09 (de regel `checked_on` van `data/prices.txt`). Controleer zelf de actuele prijzen. Je bent zelf verantwoordelijk voor elke beslissing op basis van de uitvoer. Met `--prices BESTAND` geef je een eigen prijsbestand mee.

## Onderhoud

Meld fouten via GitHub Issues van deze repository (https://github.com/qwpp7g79y4-maker/cdn-egress-bill/issues). Na een release staat de tool in sluimermodus: een nieuwe versie komt alleen bij een gemelde fout, een aanvraag of een beveiligingsprobleem, en beveiliging gaat altijd voor.

## Ondersteuning

Fouten en vragen: open een issue op GitHub (https://github.com/qwpp7g79y4-maker/cdn-egress-bill/issues). Voeg een klein synthetisch log toe, nooit een echt CloudFront-log. Beveiligingsproblemen gaan via SECURITY.md.

## Licentie

MIT OR Apache-2.0. Zie `LICENSE-MIT` en `LICENSE-APACHE`.

De licenties van de externe crates in het programma staan in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).

## Contact

PEPAI, pep_ai@icloud.com, https://www.pepai.nl, X https://x.com/pepai_nl, Ko-fi https://ko-fi.com/pepai
