#![allow(unused_imports, dead_code)]

use common::Format;
use indicatif::ProgressStyle;
use std::io::Write;

use crate::ExtractFormat;
use crate::GameVer;
use crate::StringExtractArgs;

// TODO better string location system
#[derive(serde::Serialize, alox_48::Serialize)]
struct GameString {
    location: String,
    text: String,
}

mod rmxp;

mod scripts;
use scripts::process_script_text;

#[allow(clippy::too_many_lines)] // just barely over the limit
pub fn extract(args: StringExtractArgs) {
    let StringExtractArgs {
        src,
        dest,
        game_version,
        extract_format,
        format,
        file_ext,
        scripts_dir,
    } = args;

    let format = format.unwrap_or_else(|| {
        let maybe_format = file_ext.as_deref().and_then(Format::guess_from_ext);
        let Some(format) = maybe_format else {
            // we couldn't guess the format, so error out and exit
            crate::no_format_error().exit()
        };
        format
    });

    let file_ext = file_ext.as_deref().unwrap_or(format.file_ext());

    let read_dir = match std::fs::read_dir(&src) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to read {}: {e}", src.display());
            return;
        }
    };

    #[cfg(feature = "ruby-prism")]
    let scripts_files = match &scripts_dir {
        Some(scripts_dir) => match get_script_files(scripts_dir) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to read {}: {e}", scripts_dir.display());
                return;
            }
        },
        None => vec![],
    };

    let mut entries: Vec<_> = match read_dir.collect() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("failed to read directory entry: {e}");
            return;
        }
    };
    entries.sort_by_key(std::fs::DirEntry::path);

    let mut strings = Vec::new();

    let len = entries.len();
    #[cfg(feature = "ruby-prism")]
    let len = len + scripts_files.len();

    let pb = indicatif::ProgressBar::new(len as _);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} converted",
        )
        .expect("should be valid")
        .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(50));

    for entry in entries {
        pb.inc(1);
        let path = entry.path();
        // if not a file *or* the file extension does not match what it should, print warning and continue
        if !entry.file_type().expect("couldn't get file type").is_file()
            || path.extension().is_none_or(|ext| ext != file_ext)
        {
            pb.println(format!("[WARN]: Ignoring {}", path.display()));
            continue;
        }

        let result = match game_version {
            GameVer::RPGXP => rmxp::process(&path, format, &mut strings),
        };

        match result {
            ProcessResult::Ok => {}
            ProcessResult::Err(e) => {
                pb.println(e);
                break;
            }
            ProcessResult::Unrecognized => {
                pb.println(format!("unrecognized file {}", path.display()));
            }
        }
    }

    #[cfg(feature = "ruby-prism")] // no point in handling this if prism isn't included
    process_script_files(scripts_files, &mut strings, &pb);

    let file = match std::fs::File::create(&dest) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("failed to create {}: {e}", dest.display());
            return;
        }
    };

    match extract_format {
        ExtractFormat::Json => {
            if let Err(e) = common::conv_write(strings, Format::Json, file) {
                eprintln!("failed to write to {}: {e}", dest.display());
            }
        }
        ExtractFormat::Gettext => {
            strings.dedup_by(|a, b| a.text == b.text); // gettext requires dedup'd strings
            if let Err(e) = write_gettext(strings, file) {
                eprintln!("failed to write to {}: {e}", dest.display());
            }
        }
    }
}

fn get_script_files(scripts_dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let scripts_txt_path = scripts_dir.join("_scripts.txt");
    let scripts_txt = std::fs::read_to_string(&scripts_txt_path)?;

    let scripts = scripts_txt
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|s| scripts_dir.join(s).with_extension("rb"))
        .collect();

    Ok(scripts)
}

#[cfg(feature = "ruby-prism")] // no point in handling this if prism isn't included
fn process_script_files(
    script_files: Vec<std::path::PathBuf>,
    strings: &mut Vec<GameString>,
    pb: &indicatif::ProgressBar,
) {
    for path in script_files {
        pb.inc(1);
        let script_text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to read {}: {e}", path.display());
                pb.abandon();
                return;
            }
        };

        process_script_text(script_text, strings, path.display().to_string());
    }
}

enum ProcessResult {
    Ok,
    Err(String),
    Unrecognized,
}

fn read_data<T>(path: &std::path::Path, format: Format) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + for<'de> alox_48::Deserialize<'de>
        + alox_48::Serialize,
{
    let input =
        std::fs::File::open(path).map_err(|e| format!("couldn't open {}: {e}", path.display()))?;
    let input = std::io::BufReader::new(input);

    common::conv_read(format, input).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn write_gettext(strings: Vec<GameString>, mut writer: impl std::io::Write) -> std::io::Result<()> {
    for string in strings {
        writeln!(writer, "#: {}", string.location)?;
        writeln!(writer, "msgid \"{}\"", string.text)?;
        writeln!(writer, "msgstr \"\"")?;
        writeln!(writer)?;
    }
    Ok(())
}
