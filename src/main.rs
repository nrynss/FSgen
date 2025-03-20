use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

struct AppConfig {
    input_dir: PathBuf,
    skip_dirs: Vec<String>,
    skip_files: Vec<String>,
    output_file: String,
    show_files: bool,
}

fn main() {
    // Parse command line arguments
    let config = parse_args();

    // Create output file
    let mut output_file = match fs::File::create(&config.output_file) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating output file '{}': {}", config.output_file, e);
            process::exit(1);
        }
    };

    // Get the base directory name
    let base_dir_name = config
        .input_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(".");

    // Write the base directory name to the output file
    writeln!(output_file, "{}", base_dir_name).unwrap();

    // Process the directory structure
    process_directory(
        &config.input_dir,
        &config.skip_dirs,
        &config.skip_files,
        &mut output_file,
        "",
        true,
        0,
        config.show_files,
    );

    println!(
        "Directory structure has been written to '{}'",
        config.output_file
    );
}

fn parse_args() -> AppConfig {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut input_dir = PathBuf::from(".");
    let mut skip_dirs = Vec::new();
    let mut skip_files = Vec::new();
    let mut output_file = String::from("output.txt");
    let mut show_files = true;

    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                if i + 1 < args.len() {
                    input_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing argument for -i flag");
                    print_usage();
                    process::exit(1);
                }
            }
            "-s" => {
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    skip_dirs.push(args[i].clone());
                    i += 1;
                }
            }
            "-f" => {
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    skip_files.push(args[i].clone());
                    i += 1;
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Missing argument for -o flag");
                    print_usage();
                    process::exit(1);
                }
            }
            "--no-files" => {
                show_files = false;
                i += 1;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Validate input directory
    if !input_dir.exists() || !input_dir.is_dir() {
        eprintln!("Error: '{}' is not a valid directory", input_dir.display());
        process::exit(1);
    }

    AppConfig {
        input_dir,
        skip_dirs,
        skip_files,
        output_file,
        show_files,
    }
}

fn print_usage() {
    eprintln!("Usage: fsgen [-i <input_directory>] [-s <skip_dir1> <skip_dir2> ...] [-f <skip_file1> <skip_file2> ...] [-o <output_file>] [--no-files]");
    eprintln!("  -i         Specify input directory (default: current directory)");
    eprintln!("  -s         Specify directories to skip (can be multiple)");
    eprintln!("  -f         Specify files to skip (can be multiple)");
    eprintln!("  -o         Specify output file (default: output.txt)");
    eprintln!("  --no-files Hide all files (only show directories)");
}

fn process_directory(
    dir_path: &Path,
    skip_dirs: &[String],
    skip_files: &[String],
    output: &mut fs::File,
    prefix: &str,
    _is_last: bool,
    _depth: usize,
    show_files: bool,
) {
    // Read the directory entries
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            let _ = writeln!(output, "{}├── Error reading directory: {}", prefix, e);
            return;
        }
    };

    // Collect and sort entries
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Process each entry
    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        let is_entry_last = i == entries.len() - 1;
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        // Skip directories in the skip list
        if is_dir && skip_dirs.contains(&name.to_string()) {
            continue;
        }

        // Skip files in the skip list or if show_files is false
        if !is_dir && (skip_files.contains(&name.to_string()) || !show_files) {
            continue;
        }

        // Determine the connector symbol
        let connector = if is_entry_last {
            "└── "
        } else {
            "├── "
        };

        // Write the entry
        writeln!(output, "{}{}{}", prefix, connector, name).unwrap();

        // Recursively process directories
        if is_dir {
            let new_prefix = if is_entry_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            // Using 0 as we're not tracking depth in the current implementation
            process_directory(
                &path,
                skip_dirs,
                skip_files,
                output,
                &new_prefix,
                is_entry_last,
                0,
                show_files,
            );
        }
    }
}
