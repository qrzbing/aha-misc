//! Some basic tools

use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Once},
    time::{SystemTime, UNIX_EPOCH},
};

use libafl::{HasNamedMetadata, inputs::Input, mutators::Tokens, state::HasRand};
use libafl_bolts::rands::Rand;

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

static INIT: Once = Once::new();

/// Initialize a logger
pub fn setup_logger() {
    INIT.call_once(|| {
        let _ = env_logger::try_init();
    });
}

/// Get a random token from a named metadata Tokens.
///
/// # Arguments
/// * `state` - A State implements `HasRand` and `HasNamedMetadata`
/// * `name` - Name of the metadata.
///
/// # Returns
/// `Some(Vec<u8>)`, or `None` if metadata is empty.
pub fn get_random_from_tokens<S>(state: &mut S, name: &str) -> Option<Vec<u8>>
where
    S: HasRand + HasNamedMetadata,
{
    let Some(meta) = state.named_metadata_map().get::<Tokens>(name) else {
        return None;
    };

    if meta.tokens().len() == 0 {
        return None;
    }

    let tokens_len = meta.tokens().len();
    // Old meta lifetime end

    let token_idx = state.rand_mut().below_or_zero(tokens_len);

    // A New meta lifetime
    let Some(meta) = state.named_metadata_map().get::<Tokens>(name) else {
        return None;
    };
    Some(meta.tokens()[token_idx].clone())
}
