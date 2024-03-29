#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::io::Error;
use std::path::{PathBuf};
use std::process::Command;

fn main() -> std::io::Result<()> {
    // Extract the args
    let args: Vec<String> = env::args().collect();

    // Validation check
    if args.len() < 2 {
        println!("Usage: {} [executable_path] [arg1] [arg2] ... [argN]",
                 args[0].split("\\").last().unwrap());
        return Ok(());
    }

    // Get args
    let executable_path = &args[1];
    let executable_args = &args[2..];

    // Get the latest directory
    let executable_path = get_latest_dir(&executable_path)?;

    // Run the executable
    run_executable(&executable_path, executable_args)?;
    Ok(())
}

// Get the version number from a directory
fn get_version_number(entry: &fs::DirEntry) -> Vec<u32> {
    entry.file_name()
        .to_string_lossy().as_ref()
        .split(".").collect::<Vec<_>>()
        .iter()
        .map(|x| x.parse::<u32>().unwrap())
        .collect::<Vec<_>>()
}

// Get the directory with the highest version number from a specified base directory
fn get_latest_dir(path_pattern: &str) -> Result<PathBuf, Error> {
    // Extract the folder example part of the path of a pattern like "C:\<folder_example>*\foo\bar.exe"
    let fixed_path_pattern: String = path_pattern.replace("/", "\\").to_lowercase();
    let path_parts: Vec<&str> = fixed_path_pattern.split("\\").collect();

    // Get the folder example part of the path
    let mut search_path: String = "".to_string();
    let mut versioning_pattern: String = "".to_string();
    let mut versioning_found: bool = false;
    let mut sub_path: String = "".to_string();
    for path_part in path_parts {
        if path_part.contains("*") {
            versioning_found = true;
            versioning_pattern = path_part.replace("*", "");
        } else if !versioning_found {
            if !path_part.contains(":") {
                search_path = format!("{}\\{}", search_path, path_part);
            } else {
                search_path = path_part.to_string();
            }
        } else {
            sub_path = format!("{}\\{}", sub_path, path_part);
        }
    }

    // Set the search path to the current directory if it is empty
    let current_dir = env::current_dir()?;
    if search_path.is_empty() {
        search_path = format!("{}\\", current_dir.to_str().unwrap());
    } else if !regex::Regex::new(r"^\w:").unwrap().is_match(&search_path) {
        search_path = format!("{}{}\\", current_dir.to_str().unwrap(), search_path);
    }
    search_path = search_path.to_lowercase();

    // Get the directories
    let versions_search = match fs::read_dir(&search_path) {
        Ok(versions) => versions,
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            return Err(e);
        }
    };

    let mut versions = Vec::new();
    for entry_res in versions_search {
        match entry_res {
            Ok(entry) => {
                if entry.file_type()?.is_dir() {
                    if entry.file_name().to_string_lossy().to_lowercase().contains(&versioning_pattern) {
                        versions.push(entry);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
            }
        }
    }

    // Sort the directories
    versions.sort_by(|a, b| {
        let a = get_version_number(a);
        let b = get_version_number(b);
        a.cmp(&b)
    });


    // Get the latest directory
    let latest_version: Option<String> = match versions.last() {
        Some(dir) => Some(
            dir.path()
                .into_os_string()
                .into_string()
                .map_err(|_|
                    Error::new(std::io::ErrorKind::Other, "Failed to convert to string"))?),
        None => None,
    };

    // Get the latest directory
    let latest_path: Option<PathBuf> = match latest_version {
        Some(version_folder_path) => Some(PathBuf::from(format!("{}{}",
            version_folder_path,
            sub_path
        ))),
        None => None,
    };

    // Do a final check to see if the latest path exists
    match latest_path {
        Some(path) => {
            if path.exists() {
                Ok(path)
            } else {
                Err(Error::new(std::io::ErrorKind::Other, "No valid path found"))
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "No directories found")),
    }
}

// Function to run an executable
fn run_executable(path: &PathBuf, args: &[String]) -> std::io::Result<()> {
    match Command::new(path)
        .args(args)
        .current_dir(path.parent().unwrap())
        .spawn() {
        Ok(_child) => {
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to run executable: {}", e);
            Err(Error::new(std::io::ErrorKind::Other, format!("Failed to run executable: {}", e)))
        }
    }
}