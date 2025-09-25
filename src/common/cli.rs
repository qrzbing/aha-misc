//! Some CLI Definitions

use std::path::PathBuf;

use clap::Args;

/// Base options for the fuzzer configuration.
///
/// This struct defines the common parameters needed for fuzzing operations,
/// including input/output directories and token file configuration.
#[derive(Args, Debug)]
pub struct BaseFuzzerOptions {
    /// Input corpus directory
    #[arg(
        help = "The directory to read initial inputs from ('seeds')",
        long = "in",
        required = true
    )]
    pub in_dir: PathBuf,

    /// Output corpus directory
    #[arg(long = "out", default_value = "./out")]
    pub find_corpus_dir: PathBuf,

    /// Directory to store crashes
    #[arg(long = "crashes", default_value = "./crashes")]
    pub find_crash_dir: PathBuf,

    /// Optional directory to store crash inputs
    #[arg(long = "crash_inputs")]
    pub executor_crash_dir: Option<PathBuf>,

    /// File has tokens
    #[arg(long = "token_file", default_value = "")]
    pub token_file: String,
}


/// Options for replaying test cases.
///
/// This struct defines the parameters for replaying previously generated
/// test cases, including paths to corpus files, start/end indices,
/// and debug settings.
#[derive(Args, Debug)]
pub struct ReplayOptions {
    /// Replay corpus dir, could be dirs
    #[arg(long = "replay-dir", action = clap::ArgAction::Append)]
    pub replay_dirs: Option<Vec<String>>,

    /// Replay corpus file, could be files
    #[arg(long = "replay-file", action = clap::ArgAction::Append)]
    pub replay_files: Option<Vec<String>>,

    /// Start replaying from this index (0-based, inclusive)
    #[arg(long, default_value = "0")]
    pub start: usize,

    /// End replaying at this index (0-based, exclusive), defaults to all files if not specified
    #[arg(long)]
    pub end: Option<usize>,

    /// Enable debug mode
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}

impl ReplayOptions {
    /// Get the list of files to replay.
    ///
    /// # Returns
    /// A vector of PathBuf objects pointing to the files that should be replayed.
    ///
    /// # Description
    /// This method processes both directories and individual files specified in the options.
    /// For directories, it includes all files within those directories. It skips any paths
    /// that don't exist or are not of the expected type.
    pub fn get_replay_files(&self) -> Vec<PathBuf> {
        let mut to_replay_files: Vec<PathBuf> = vec![];

        // Process directories
        if let Some(dirs) = &self.replay_dirs {
            for dir_str in dirs {
                let dir = PathBuf::from(dir_str);
                if dir.is_dir() {
                    for entry in std::fs::read_dir(&dir).unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() {
                            to_replay_files.push(path);
                        }
                    }
                } else {
                    eprintln!("Warning: Skipping non-directory path: {:?}", dir);
                }
            }
        }

        // Process individual files
        if let Some(files) = &self.replay_files {
            for file_str in files {
                let file = PathBuf::from(file_str);
                if file.is_file() {
                    to_replay_files.push(file);
                } else {
                    eprintln!("Warning: Skipping non-file path: {:?}", file);
                }
            }
        }

        to_replay_files
    }

    /// Get the end index for replay.
    ///
    /// # Returns
    /// The end index if specified, or usize::MAX to indicate no limit.
    pub fn get_end(&self) -> usize {
        self.end.unwrap_or(usize::MAX)
    }
}
