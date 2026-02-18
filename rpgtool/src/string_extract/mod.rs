#![allow(unused_imports, dead_code)]

use common::Format;

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

fn write_gettext(strings: Vec<GameString>, mut writer: impl std::io::Write) -> std::io::Result<()> {
    use std::io::Write;

    for string in strings {
        writeln!(writer, "#: {}", string.location)?;
        writeln!(writer, "msgid \"{}\"", string.text)?;
        writeln!(writer, "msgstr \"\"")?;
        writeln!(writer)?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn extract(args: StringExtractArgs) {
    use std::io::Write;

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

    let mut strings = Vec::new();

    let mut entries: Vec<_> = match read_dir.collect() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("failed to read directory entry: {e}");
            return;
        }
    };
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        // if not a file *or* the file extension does not match what it should, print warning and continue
        if !entry.file_type().expect("couldn't get file type").is_file()
            || path.extension().is_none_or(|ext| ext != file_ext)
        {
            println!("[WARN]: Ignoring {}", path.display());
            continue;
        }

        let result = match game_version {
            GameVer::RPGXP => rmxp::process(&path, format, &mut strings),
        };

        match result {
            ProcessResult::Ok => {}
            ProcessResult::Err(e) => {
                println!("{e}");
                break;
            }
            ProcessResult::Unrecognized => {
                println!("unrecognized file {}", path.display());
                break;
            }
        }
    }

    #[cfg(feature = "ruby-prism")] // no point in handling this if prism isn't included
    if let Some(scripts_dir) = scripts_dir {
        let scripts_txt_path = scripts_dir.join("_scripts.txt");
        let scripts_txt = match std::fs::read_to_string(&scripts_txt_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to read {}: {e}", scripts_txt_path.display());
                return;
            }
        };

        for name in scripts_txt.lines() {
            let trimmed_name = name.trim();
            if trimmed_name.is_empty() || trimmed_name.starts_with('#') {
                continue;
            }

            let script_path = scripts_dir.join(trimmed_name).with_extension("rb");
            let script_text = match std::fs::read_to_string(&script_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("failed to read {}: {e}", script_path.display());
                    continue;
                }
            };

            process_script_text(script_text, &mut strings, script_path.display().to_string());
        }
    }

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
