use clap::{Arg, Command};
use colored::*;
use std::fs;
use std::path::Path;

fn main() {
    let matches = Command::new("peek")
        .version("0.1.0")
        .author("Tikrack <tikrackcode@gmail.com>")
        .about("A modern ls replacement written in Rust")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .help("Show file sizes"),
        )
        .get_matches();

    let current_dir = ".";
    let show_size = matches.is_present("size");

    match fs::read_dir(current_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_name = entry.file_name().to_string_lossy().to_string();

                    if file_name.starts_with('.') {
                        continue;
                    }

                    if show_size {
                        if let Ok(metadata) = entry.metadata() {
                            let size = metadata.len();
                            let colored_name = format_filename(&path);
                            println!("{:>8}  {}", format_size(size), colored_name);
                        }
                    } else {
                        print!("{}  ", format_filename(&path));
                    }
                }
            }

            if !show_size {
                println!();
            }
        }
        Err(e) => eprintln!("Error reading directory: {}", e),
    }
}

fn format_filename(path: &Path) -> ColoredString {
    if path.is_dir() {
        path.file_name().unwrap().to_string_lossy().blue().bold()
    } else {
        path.file_name().unwrap().to_string_lossy().white()
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{} B", bytes),
    }
}
