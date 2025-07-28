use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

/// Parse passed parameters to the CLI command. It can be either a folder
/// or a file (although folder is not supported right now).
pub(super) fn get_file_from_args() -> (String, PathBuf) {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Please specify a filename to open");
        exit(1);
    }

    let passed_path = args.last().expect("Could not read passed path");
    let mut canonical_path = fs::canonicalize(passed_path).expect("Could not read passed path");

    let file_string = fs::read_to_string(&canonical_path).expect("Could not open file");

    canonical_path.pop();

    return (file_string, canonical_path);
}
