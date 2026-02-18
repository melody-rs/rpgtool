#![allow(
    clippy::struct_field_names,
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation
)]

mod nil_padded;
pub use nil_padded::NilPadded;

mod rgss_structs;
pub use rgss_structs::*;

mod shared;
pub use shared::*;

pub mod rmxp;

use crate::{GameVer, StructuredArgs};

use common::Format;
use indicatif::ProgressStyle;
use rayon::prelude::*;

macro_rules! fail {
    ($pb:expr, $fail_on_error:expr) => {
        if $fail_on_error {
            $pb.abandon();
            return None;
        }
    };
}

#[allow(clippy::too_many_lines)]
pub fn convert(args: StructuredArgs) {
    let StructuredArgs {
        src,
        dest,
        game_version,
        format,
        fail_on_error,
        input_file_ext,
        output_file_ext,
        single_thread,
        thread_count,
    } = args;

    if let Some(count) = thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(count)
            .build_global()
            .expect("failed to build thread pool");
    }

    let [from, to] = match format.as_deref() {
        Some(&[from, to]) => [from, to],
        // try to guess format if not specified
        None => {
            let maybe_from = input_file_ext.as_deref().and_then(Format::guess_from_ext);
            let maybe_to = output_file_ext.as_deref().and_then(Format::guess_from_ext);
            let Some((from, to)) = maybe_from.zip(maybe_to) else {
                crate::no_format_error().exit()
            };
            [from, to]
        }
        _ => unreachable!(), // we enforce the number of values in clap
    };

    let input_file_ext = input_file_ext.as_deref().unwrap_or(from.file_ext());
    let output_file_ext = output_file_ext.as_deref().unwrap_or(to.file_ext());

    // try and create the destination dir, and if it doesn't exist then error out
    if let Err(e) = std::fs::create_dir(&dest)
        && e.kind() != std::io::ErrorKind::AlreadyExists
    {
        eprintln!("failed to create destination directory: {e}");
        return;
    }

    let read_dir = match std::fs::read_dir(&src) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to read {}: {e}", src.display());
            return;
        }
    };

    let mut entries: Vec<_> = if fail_on_error {
        match read_dir.collect() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("failed to read directory entry: {e}");
                return;
            }
        }
    } else {
        read_dir.filter_map(Result::ok).collect()
    };
    entries.sort_by_key(std::fs::DirEntry::path);

    let pb = indicatif::ProgressBar::new(entries.len() as _);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} converted",
        )
        .expect("should be valid")
        .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(50));

    let entry_fn = |entry: &std::fs::DirEntry| {
        pb.inc(1);
        let src_path = entry.path();
        // if not a file *or* the file extension does not match what it should, print warning and continue
        if !entry.file_type().expect("couldn't get file type").is_file()
            || src_path.extension().is_none_or(|ext| ext != input_file_ext)
        {
            pb.println(format!("[WARN]: Ignoring {}", src_path.display()));
            return Some(());
        }

        let filename = src_path.file_name().expect("entry should have a file name");
        let dest_path = dest.join(filename).with_extension(output_file_ext);

        let result = match game_version {
            GameVer::RPGXP => convert_xp(&src_path, &dest_path, to, from),
        };

        match result {
            ConvertResult::Ok => {}
            ConvertResult::Err(e) => {
                pb.println(e);
                fail!(pb, fail_on_error);
            }
            ConvertResult::Unrecognized => {
                pb.println(format!("unrecognized file {}", src_path.display()));
                fail!(pb, fail_on_error);

                if let Err(e) = convert_data::<common::Value>(&src_path, &dest_path, to, from) {
                    pb.println(e);
                    fail!(pb, fail_on_error);
                }
            }
        }

        Some(())
    };

    let result = if single_thread {
        entries.iter().try_for_each(entry_fn)
    } else {
        entries.par_iter().try_for_each(entry_fn)
    };

    if result.is_some() {
        pb.finish();
    }
}

enum ConvertResult {
    Ok,
    Err(String),
    Unrecognized,
}

fn convert_xp(
    src_path: &std::path::Path,
    dest_path: &std::path::Path,
    to: Format,
    from: Format,
) -> ConvertResult {
    let prefix = src_path.file_prefix().expect("there should be a prefix");
    let Some(filename) = prefix.to_str() else {
        return ConvertResult::Err(format!("{} is not valid UTF-8", prefix.display()));
    };

    let result = match filename {
        "Actors" => convert_data::<rmxp::Actors>(src_path, dest_path, to, from),
        "Animations" => convert_data::<rmxp::Animations>(src_path, dest_path, to, from),
        "Armors" => convert_data::<rmxp::Armors>(src_path, dest_path, to, from),
        "Classes" => convert_data::<rmxp::Classes>(src_path, dest_path, to, from),
        "Enemies" => convert_data::<rmxp::Enemies>(src_path, dest_path, to, from),
        "Items" => convert_data::<rmxp::Items>(src_path, dest_path, to, from),
        "Skills" => convert_data::<rmxp::Skills>(src_path, dest_path, to, from),
        "States" => convert_data::<rmxp::States>(src_path, dest_path, to, from),
        "System" => convert_data::<rmxp::System>(src_path, dest_path, to, from),
        "Tilesets" => convert_data::<rmxp::Tilesets>(src_path, dest_path, to, from),
        "Troops" => convert_data::<rmxp::Troops>(src_path, dest_path, to, from),
        "Weapons" => convert_data::<rmxp::Weapons>(src_path, dest_path, to, from),
        "MapInfos" => convert_data::<shared::MapInfos>(src_path, dest_path, to, from),
        "CommonEvents" => convert_data::<shared::CommonEvents>(src_path, dest_path, to, from),
        _ if filename.starts_with("Map") => {
            convert_data::<rmxp::Map>(src_path, dest_path, to, from)
        }
        "Scripts" | "xScripts" => {
            convert_data::<Vec<shared::Script>>(src_path, dest_path, to, from)
        }
        _ => return ConvertResult::Unrecognized,
    };

    match result {
        Ok(()) => ConvertResult::Ok,
        Result::Err(e) => ConvertResult::Err(e),
    }
}

fn convert_data<T>(
    src_path: &std::path::Path,
    dest_path: &std::path::Path,
    to: Format,
    from: Format,
) -> Result<(), String>
where
    T: for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + for<'de> alox_48::Deserialize<'de>
        + alox_48::Serialize,
{
    let input = std::fs::File::open(src_path)
        .map_err(|e| format!("couldn't open {}: {e}", src_path.display()))?;
    let input = std::io::BufReader::new(input);

    let value: T = common::conv_read(from, input)
        .map_err(|e| format!("failed to parse {}: {e}", src_path.display()))?;

    let output = std::fs::File::create(dest_path)
        .map_err(|e| format!("couldn't open {}: {e}", dest_path.display()))?;
    let output = std::io::BufWriter::new(output);

    common::conv_write(value, to, output)
        .map_err(|e| format!("failed to convert {}: {e}", src_path.display()))?;

    Ok(())
}
