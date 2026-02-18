use common::Format;
use indicatif::ProgressStyle;
use rayon::prelude::*;

use super::ConvArgs;

pub fn convert(args: ConvArgs) {
    let ConvArgs {
        src,
        dest,
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

        if let Err(e) = convert_data::<common::Value>(&src_path, &dest_path, to, from) {
            pb.println(e);
            if fail_on_error {
                pb.abandon();
                return None;
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
