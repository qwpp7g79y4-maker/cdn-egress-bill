#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};

use cdn_egress_bill::{
    build_report, default_edges, default_prices, load_edges, load_logs, load_prices, path_text,
    price_age_warning, render_json, render_markdown, render_text, today_utc, Error, OVERFLOW,
};
use clap::{Parser, ValueEnum};

#[cfg(cdn_git_sha)]
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("CDN_GIT_SHA"), ")");
#[cfg(not(cdn_git_sha))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "cdn-egress-bill",
    bin_name = "cdn-egress-bill",
    version = VERSION,
    about = "Schat CloudFront-egressiekosten uit standaardlogbestanden (offline, geen AWS-API)."
)]
struct Cli {
    /// Prijsbestand. Standaard: de ingebouwde tarieven.
    #[arg(long)]
    prices: Option<PathBuf>,
    /// Edge-kaart. Standaard: de ingebouwde IATA-codes.
    #[arg(long)]
    edges: Option<PathBuf>,
    /// text of json.
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Alleen deze maand (YYYY-MM).
    #[arg(long)]
    month: Option<String>,
    /// Peildatum (YYYY-MM-DD) voor de ouderdom van de prijzen. Standaard: vandaag (UTC).
    #[arg(long)]
    as_of: Option<String>,
    /// Markdown in plaats van tekst of JSON.
    #[arg(long, conflicts_with = "format")]
    md: bool,
    /// Schrijf het rapport naar dit bestand in plaats van naar stdout.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Alleen het resultaat en fouten; geen waarschuwingen.
    #[arg(long, conflicts_with = "verbose")]
    quiet: bool,
    /// Voortgang op stderr.
    #[arg(long)]
    verbose: bool,
    /// Exit 3 als er onberekende bytes of verzoeken zijn.
    #[arg(long)]
    strict: bool,
    /// CloudFront-standaardlogbestanden (tekst of gzip).
    #[arg(required = true)]
    logs: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() {
    cdn_egress_bill::install_sigint_handler();
    let code = match run() {
        Ok(code) => code,
        Err(err) => {
            report_stderr(&err.to_string());
            2
        }
    };
    if code != 0 {
        std::process::exit(i32::from(code));
    }
}

// `eprintln!` panikeert als stderr niet te schrijven is.
fn report_stderr(message: &str) {
    let _ = writeln!(std::io::stderr(), "{message}");
}

/// De clap-tekst voor een gebruiksfout, zonder dat de invoer van de gebruiker een regel kan bijmaken.
///
/// Een argument dat geen UTF-8 is, gaat als bytes-ontsnapping mee (`path_text`).
///
/// clap haalt ANSI-reeksen uit wat het herhaalt, dus de ruwe waarde is in de tekst niet
/// terug te vinden. Daarom vraagt dit clap om dezelfde fout voor dezelfde argumenten
/// met elk onveilig teken al als `\u....` geschreven: elke regeleinde in die tekst is
/// dan van clap zelf.
fn clap_message(err: &clap::Error) -> String {
    let args: Vec<std::ffi::OsString> = std::env::args_os()
        .map(|arg| match arg.to_str() {
            Some(text) => std::ffi::OsString::from(Error::new(text).to_string()),
            // Geen UTF-8: de bytes als `\xff`, nooit een rauwe regeleinde.
            None => std::ffi::OsString::from(path_text(Path::new(&arg))),
        })
        .collect();
    let text = match Cli::try_parse_from(args) {
        Err(own) if own.kind() == err.kind() => own.render().to_string(),
        // clap meldt een niet-UTF-8-argument zonder de waarde te herhalen: die tekst is veilig.
        _ if err.kind() == clap::error::ErrorKind::InvalidUtf8 => err.render().to_string(),
        // Onverwacht: een andere fout voor de ontsnapte argumenten. Dan niets vertrouwen.
        _ => Error::new(err.render().to_string()).to_string(),
    };
    // Wat aan onveilige tekens overblijft komt ook ontsnapt; alleen de regeleinden van clap blijven.
    let mut out = String::with_capacity(text.len());
    for (index, line) in text.trim_end_matches('\n').split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&Error::new(line).to_string());
    }
    out
}

fn run() -> Result<u8, Error> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            use clap::error::ErrorKind;
            // clap negeert schrijffouten bij --help en --version en eindigt met 0.
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                write_stdout(&err.render().to_string())?;
                return Ok(0);
            }
            report_stderr(&clap_message(&err));
            std::process::exit(err.exit_code())
        }
    };
    let prices = match &cli.prices {
        Some(path) => load_prices(&read_text(path)?)?,
        None => default_prices()?,
    };
    let reference = match &cli.as_of {
        Some(date) if cdn_egress_bill::is_date(date) => date.clone(),
        Some(date) => return Err(Error::new(format!("--as-of is geen datum: {date}"))),
        None => today_utc(),
    };
    if !cli.quiet {
        if let Some(warning) = price_age_warning(&prices.checked_on, &reference) {
            report_stderr(&warning);
        }
    }
    let progress = |message: String| {
        if cli.verbose {
            report_stderr(&Error::new(message).to_string());
        }
    };
    progress(format!(
        "prijzen gecontroleerd op {}, peildatum {reference}",
        prices.checked_on
    ));
    let edges = match &cli.edges {
        Some(path) => load_edges(&read_text(path)?)?,
        None => default_edges()?,
    };
    for log in &cli.logs {
        progress(format!("log lezen: {}", path_text(log)));
    }
    let records = load_logs(&cli.logs)?;
    progress(format!("{} records gelezen", records.len()));
    let report = build_report(&prices, &edges, &records, cli.month.as_deref())
        .map_err(|err| name_logs(err, &cli.logs))?;
    progress(format!("{} maand(en) berekend", report.months.len()));
    let text = if cli.md {
        render_markdown(&report)
    } else {
        match cli.format {
            Format::Text => render_text(&report),
            Format::Json => render_json(&report),
        }
    };
    match &cli.output {
        Some(path) => {
            if let Some(code) = write_output_file(path, &text)? {
                return Ok(code);
            }
        }
        None => write_stdout(&text)?,
    }
    let unpriced = report
        .months
        .iter()
        .any(|month| month.unpriced_bytes > 0 || month.unpriced_requests > 0);
    if cli.strict && unpriced {
        report_stderr("onberekend verkeer");
        return Ok(3);
    }
    Ok(0)
}

/// Write the report next to `path` and rename it over `path`: the file is the old
/// one or the whole new one. A symlink at `path` is replaced, never followed.
/// `Some(130)` when a SIGINT arrived while publishing (the report is complete then).
fn write_output_file(path: &Path, text: &str) -> Result<Option<u8>, Error> {
    let fail =
        |err: &dyn std::fmt::Display| Error::new(format!("--output {}: {err}", path_text(path)));
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.is_dir() {
            return Err(fail(&"is a directory"));
        }
    }
    let name = path
        .file_name()
        .ok_or_else(|| fail(&"no file name"))?
        .to_os_string();
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    cdn_egress_bill::hold_sigint();
    let result = publish(&dir, &name, path, text);
    if cdn_egress_bill::release_sigint() {
        report_stderr(cdn_egress_bill::INTERRUPTED_LINE);
        return result
            .map(|()| Some(cdn_egress_bill::EXIT_INTERRUPTED))
            .map_err(|err| fail(&err));
    }
    result.map(|()| None).map_err(|err| fail(&err))
}

fn publish(
    dir: &Path,
    name: &std::ffi::OsStr,
    path: &Path,
    text: &str,
) -> Result<(), std::io::Error> {
    let mut attempt = 0u32;
    let (mut file, temp) = loop {
        let mut temp_name = std::ffi::OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(".{}.{attempt}.tmp", std::process::id()));
        let temp = dir.join(temp_name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => break (file, temp),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists && attempt < 100 => {
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .and_then(|()| {
            drop(file);
            std::fs::rename(&temp, path)
        });
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

fn write_stdout(text: &str) -> Result<(), Error> {
    // `std::io::stdout` telt EBADF als een gelukte schrijfactie. Een kopie
    // van de descriptor geeft een alleen-lezen stdout wel door.
    let file = duplicate_stdout()?;
    // Een gesloten stdout is vóór main al vervangen door /dev/null, lees-schrijf.
    // Op Linux legt `stdout_probe` vóór die vervanging vast of fd 1 dicht was;
    // `1<>/dev/null` en `>/dev/null` blijven zo gewone uitvoer.
    #[cfg(all(unix, target_os = "linux"))]
    if cdn_egress_bill::stdout_was_closed_at_start() {
        return Err(closed_stdout_error());
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    if stdout_was_closed(&file)? {
        return Err(closed_stdout_error());
    }
    #[cfg(windows)]
    {
        // Op een Windows-console schrijft de standaarduitvoer UTF-16.
        // De kopie hierboven heeft een gesloten handle al geweigerd.
        if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            drop(file);
            return write_all_stdout(std::io::stdout().lock(), text);
        }
    }
    write_all_stdout(file, text)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn stdout_was_closed(mut file: &std::fs::File) -> Result<bool, Error> {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let ours = file
        .metadata()
        .map_err(|err| Error::new(format!("stdout: {err}")))?;
    let null = std::fs::File::open("/dev/null")
        .and_then(|null| null.metadata())
        .map_err(|err| Error::new(format!("stdout: {err}")))?;
    if ours.dev() != null.dev() || ours.ino() != null.ino() {
        return Ok(false);
    }
    let mut buf = [0u8; 1];
    // Ok(0) is EOF van een leesbare /dev/null (`1<>/dev/null`): gewone uitvoer.
    // Een echt gesloten fd 1 is na de start niet meer van dat geval te onderscheiden.
    Ok(file.read(&mut buf).is_ok_and(|n| n > 0))
}

#[cfg(unix)]
fn closed_stdout_error() -> Error {
    let err = std::io::Error::from_raw_os_error(9);
    Error::new(format!("stdout: {err}"))
}

fn write_all_stdout(mut out: impl Write, text: &str) -> Result<(), Error> {
    out.write_all(text.as_bytes())
        .map_err(|err| Error::new(format!("stdout: {err}")))?;
    out.flush()
        .map_err(|err| Error::new(format!("stdout: {err}")))?;
    Ok(())
}

#[cfg(unix)]
fn duplicate_stdout() -> Result<std::fs::File, Error> {
    use std::os::fd::AsFd;
    std::io::stdout()
        .as_fd()
        .try_clone_to_owned()
        .map(std::fs::File::from)
        .map_err(|err| Error::new(format!("stdout: {err}")))
}

#[cfg(windows)]
fn duplicate_stdout() -> Result<std::fs::File, Error> {
    use std::os::windows::io::AsHandle;
    std::io::stdout()
        .as_handle()
        .try_clone_to_owned()
        .map(std::fs::File::from)
        .map_err(|err| Error::new(format!("stdout: {err}")))
}

/// A bare overflow names the logs that took part, never their text.
fn name_logs(err: Error, logs: &[PathBuf]) -> Error {
    if err.message != OVERFLOW {
        return err;
    }
    let names: Vec<String> = logs.iter().map(|log| path_text(log)).collect();
    Error::new(format!("{}: {OVERFLOW}", names.join(", ")))
}

fn read_text(path: &Path) -> Result<String, Error> {
    use std::io::Read;
    let fail = |err: &dyn std::fmt::Display| Error::new(format!("{}: {err}", path_text(path)));
    // Eerst stat: een fifo openen blokkeert zolang er geen schrijver is.
    let before = std::fs::metadata(path).map_err(|err| fail(&err))?;
    if !before.is_file() {
        return Err(fail(&"geen gewoon bestand"));
    }
    let file = std::fs::File::open(path).map_err(|err| fail(&err))?;
    let meta = file.metadata().map_err(|err| fail(&err))?;
    if !meta.is_file() {
        return Err(fail(&"geen gewoon bestand"));
    }
    cdn_egress_bill::check_file_size(path, meta.len())?;
    let mut bytes = Vec::new();
    file.take(meta.len())
        .read_to_end(&mut bytes)
        .map_err(|err| fail(&err))?;
    String::from_utf8(bytes).map_err(|err| fail(&err))
}
