#![warn(clippy::unwrap_used, clippy::pedantic, rust_2018_idioms)]

extern crate rpgtool_common as common;

use clap::{CommandFactory, Parser};
use common::Format;
use std::path::PathBuf;

mod conv;
mod pack;
mod string_extract;
mod structured;
mod unpack;

#[must_use]
pub fn no_format_error() -> clap::Error {
    let mut command = Cli::command();
    command.error(
        clap::error::ErrorKind::DisplayHelp,
        "unable to determine conversion formats, please specify with --format",
    )
}

/// Utility for working with RPG Maker XP - VX Ace projects.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Dynamically convert an entire project to another data format.
    Convert(ConvArgs),
    /// Unpack a scripts file into a directory.
    Unpack(UnpackScriptArgs),
    /// Pack a scripts directory back into a file.
    Pack(PackScriptArgs),
    /// Rigidly convert an entire project to another data format.
    ///
    /// This uses a more rigid structured representation of game data instead
    /// of a loose format.
    ///
    /// Unknown game files are converted using the normal structured format.
    Structured(StructuredArgs),
    /// Generate completions for the specified shell.
    Completions(CompletionArgs),
    /// Extract all the strings from game data.
    StringExtract(StringExtractArgs),
}

#[derive(clap::Args)]
struct CompletionArgs {
    shell: clap_complete::Shell,
}

#[derive(clap::Args)]
struct ConvArgs {
    /// The source directory.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    src: PathBuf,
    /// The destination directory.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    dest: PathBuf,
    /// The formats to convert from/to.
    ///
    /// Input comes first.
    ///
    /// Required if the format cannot be determined via file extensions.
    #[arg(long, visible_short_alias = 'f', number_of_values = 2)]
    format: Option<Vec<Format>>,
    /// Exit on error instead of ignoring it.
    #[arg(long = "fail-fast")]
    fail_on_error: bool,
    #[arg(long = "output-ext")]
    /// The file extension every input file uses.
    ///
    /// Optional, does not have to be specified.
    input_file_ext: Option<PathBuf>,
    #[arg(long = "input-ext")]
    /// The file extension every output file uses.
    ///
    // Optional, does not have to be specified.
    output_file_ext: Option<PathBuf>,
    /// Run with a single thread instead of multiple threads.
    #[arg(long = "single-thread")]
    single_thread: bool,
    #[arg(long = "thread-count", conflicts_with = "single_thread")]
    thread_count: Option<usize>,
}

#[derive(clap::Args)]
struct StructuredArgs {
    /// The source directory.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    src: PathBuf,
    /// The destination directory.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    dest: PathBuf,
    /// The game version to use.
    game_version: GameVer,
    /// The formats to convert from/to.
    ///
    /// Input comes first.
    ///
    /// Required if the format cannot be determined via file extensions.
    #[arg(long, visible_short_alias = 'f', number_of_values = 2)]
    format: Option<Vec<Format>>,
    /// Exit on error instead of ignoring it.
    #[arg(long = "fail-fast")]
    fail_on_error: bool,
    #[arg(long = "output-ext")]
    /// The file extension every input file uses.
    ///
    /// Optional, does not have to be specified.
    input_file_ext: Option<PathBuf>,
    #[arg(long = "input-ext")]
    /// The file extension every output file uses.
    ///
    // Optional, does not have to be specified.
    output_file_ext: Option<PathBuf>,
    /// Run with a single thread instead of multiple threads.
    #[arg(long = "single-thread")]
    single_thread: bool,
    #[arg(long = "thread-count", conflicts_with = "single_thread")]
    thread_count: Option<usize>,
}

#[derive(clap::Args)]
struct StringExtractArgs {
    /// The source directory.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    src: PathBuf,
    /// The destination file.
    #[arg(value_hint = clap_complete::ValueHint::FilePath)]
    dest: PathBuf,
    /// The game version to use.
    game_version: GameVer,
    /// The format to extract strings to.
    extract_format: ExtractFormat,
    /// The data file format to read.
    ///
    /// Required if the format cannot be determined via file extensions.
    #[arg(long, visible_short_alias = 'f')]
    format: Option<Format>,
    /// The file extension every game file uses.
    ///
    /// Optional, does not have to be specified.
    file_ext: Option<PathBuf>,
    /// The directory to read unpacked scripts from.
    ///
    /// Optional, does not have to be specified.
    #[cfg(feature = "ruby-prism")]
    #[arg(long, visible_short_alias = 's')]
    scripts_dir: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum ExtractFormat {
    /// JSON file
    Json,
    /// GNU Gettext
    Gettext,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(clap::ValueEnum, Clone)]
enum GameVer {
    RPGXP,
}

#[derive(clap::Args)]
struct UnpackScriptArgs {
    /// The packed script file.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    file: PathBuf,
    /// The directory containing script files.
    #[arg(value_hint = clap_complete::ValueHint::FilePath)]
    directory: PathBuf,
}

#[derive(clap::Args)]
struct PackScriptArgs {
    /// The directory containing script files.
    #[arg(value_hint = clap_complete::ValueHint::DirPath)]
    directory: PathBuf,
    /// The packed script file.
    #[arg(value_hint = clap_complete::ValueHint::FilePath)]
    file: PathBuf,
}

#[allow(unused)]
fn main() {
    match Cli::parse().command {
        Commands::Convert(conv_args) => conv::convert(conv_args),
        Commands::Pack(script_args) => pack::pack(script_args),
        Commands::Unpack(script_args) => unpack::unpack(script_args),
        Commands::Structured(structured_args) => structured::convert(structured_args),
        Commands::Completions(CompletionArgs { shell }) => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_owned();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::StringExtract(extract_args) => string_extract::extract(extract_args),
    }
}
