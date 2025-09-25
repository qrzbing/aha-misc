//! Some basic tools

use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use libafl::inputs::Input;

use crate::common::cli::ReplayOptions;

/// Read a file and convert each line to a vector of bytes.
///
/// # Arguments
/// * `file_path` - The path to the file to read.
///
/// # Returns
/// A `Result` containing a vector of byte vectors (`Vec<Vec<u8>>`), where each inner vector
/// represents a line from the file as bytes. Returns an empty vector if the path is empty.
/// Returns an `io::Error` if file operations fail.
pub fn read_file_as_vecs<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<Vec<u8>>> {
    let path = file_path.as_ref();

    if path.as_os_str().is_empty() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut result = Vec::new();

    for line in reader.lines() {
        let line = line?;
        result.push(line.into_bytes());
    }

    Ok(result)
}

/// Save input queue to a directory.
///
/// # Arguments
/// * `save_queue_dir` - The root directory to save the inputs.
/// * `input_queue` - The input queue to save.
///
/// # Returns
/// Return path of subdir if success, None if failed.
pub fn save_queue<I>(
    save_queue_dir: &Path,
    input_queue: &Arc<Mutex<VecDeque<I>>>,
) -> Option<PathBuf>
where
    I: Input,
{
    if save_queue_dir.as_os_str().is_empty() {
        log::debug!("Crash directory is not set. Skipping saving inputs.");
        return None;
    }

    let queue = match input_queue.lock() {
        Ok(queue) => queue,
        Err(e) => {
            log::error!("Failed to lock input queue: {}", e);
            return None;
        }
    };

    if queue.is_empty() {
        log::debug!("No inputs to save");
        return None;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs();

    let save_subdir = save_queue_dir.join(format!("crash_{}", timestamp));

    if let Err(e) = std::fs::create_dir_all(&save_subdir) {
        log::error!("Failed to create crash subdirectory: {}", e);
        return None;
    }

    for (idx, input) in queue.iter().enumerate() {
        let file_path = save_subdir.join(format!("{}.inp", idx));
        if let Err(e) = input.to_file(file_path) {
            log::error!("Failed to write input to file: {}", e);
            continue;
        }
    }

    log::debug!(
        "Saved {} inputs to directory: {:?}",
        queue.len(),
        save_subdir
    );

    Some(save_subdir)
}

/// Replay inputs from files using the provided harness function.
///
/// # Arguments
/// * `harness` - A mutable function that accepts a `BytesInput` reference and produces some result.
/// * `replay_cfg` - Configuration options for the replay process.
///
/// # Returns
/// `Ok(())` if all files were replayed successfully, or an `Error` if any file operations fail.
///
/// # Description
/// This function iterates through a list of files specified in the replay configuration,
/// reads each file's contents, converts them to `BytesInput` objects, and passes them
/// to the provided harness function. It respects the start and end indices in the configuration
/// and provides progress updates every 100 files processed.
pub fn replay<I, F, R>(
    harness: &mut F,
    replay_cfg: &ReplayOptions,
) -> Result<(), Box<dyn std::error::Error>>
where
    I: Input,
    F: FnMut(&I) -> R,
{
    // 函数体保持不变
    let mut count = 0;

    let end = replay_cfg.get_end();

    for (index, file) in replay_cfg.get_replay_files().iter().enumerate() {
        if index < replay_cfg.start || index >= end {
            continue;
        }

        if replay_cfg.debug {
            println!("Replaying file: {}", file.display());
        }

        // let data = fs::read(file)?;
        let input = I::from_file(file).unwrap();
        (*harness)(&input);

        count += 1;
        if count % 100 == 0 {
            println!("Send {} messages", count);
        }
    }

    println!("Send {} messages", count);

    Ok(())
}

/// Display the content of a seed file.
pub fn show_seed<I>(filename: &PathBuf)
where
    I: Input,
{
    if let Ok(input) = I::from_file(filename) {
        println!("Input from file {}: {:?}", filename.display(), input);
    } else {
        println!("Failed to read input from file {}", filename.display());
    }
}
