use clap::{Arg, Command};
use colored::{Color, Colorize, ColoredString};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use glob::Pattern;

#[derive(Serialize, Deserialize, Default)]
struct Config {
    dir_color: Option<String>,
}

fn main() {
    let matches = Command::new("peek")
        .version("0.1.0")
        .author("Tikrack <tikrackcode@gmail.com>")
        .about("A modern ls replacement written in Rust")
        .arg(Arg::new("size").short('s').long("size").help("Show file sizes").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("all").short('a').long("all").help("Show hidden files").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("long").short('l').long("long").help("Long listing format").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("tree").short('t').long("tree").help("Show directory structure as tree").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("depth").long("depth").value_name("N").help("Maximum depth for tree view").value_parser(clap::value_parser!(usize)))
        .arg(Arg::new("dir-color").long("dir-color").help("Set directory color as hex code (e.g. FF0000 or #FF0000)"))
        .arg(Arg::new("pattern").short('p').long("pattern").value_name("PATTERN").help("Filter files by glob pattern (e.g. *.rs)"))
        .get_matches();

    let config_path = get_config_path().expect("Cannot find home directory");
    let mut config = read_config(&config_path).unwrap_or_default();

    if let Some(color_code) = matches.get_one::<String>("dir-color") {
        config.dir_color = Some(normalize_hex_color(color_code));
        write_config(&config_path, &config).expect("Failed to write config");
    }

    let show_size = matches.get_flag("size");
    let show_all = matches.get_flag("all");
    let show_long = matches.get_flag("long");
    let show_tree = matches.get_flag("tree");
    let max_depth = matches.get_one::<usize>("depth").copied();
    let pattern = matches.get_one::<String>("pattern").map(|s| s.to_string());
    let dir_color_rgb = config.dir_color.as_ref().and_then(|hex| parse_hex_color(hex));
    let compiled_pattern = pattern.as_ref().and_then(|p| Pattern::new(p).ok());

    if show_tree {
        print_tree(Path::new("."), "".to_string(), dir_color_rgb, 0, max_depth, show_all, compiled_pattern.as_ref());
        return;
    }

    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_name = entry.file_name().to_string_lossy().to_string();

                    if !show_all && file_name.starts_with('.') {
                        continue;
                    }

                    if let Some(ref pattern) = compiled_pattern {
                        if !pattern.matches_path(&path) {
                            continue;
                        }
                    }

                    if show_long {
                        if let Ok(metadata) = entry.metadata() {
                            let permissions = metadata.mode();
                            let nlink = metadata.nlink();
                            let uid = metadata.uid();
                            let gid = metadata.gid();
                            let size = metadata.len();
                            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                            let datetime: DateTime<Local> = modified.into();

                            let perm_string = format_permissions(permissions);

                            println!(
                                "{} {:>2} {:>5} {:>5} {:>8} {} {}",
                                perm_string,
                                nlink,
                                uid,
                                gid,
                                size,
                                datetime.format("%Y-%m-%d %H:%M:%S"),
                                format_filename(&path, dir_color_rgb)
                            );
                        }
                    } else if show_size {
                        if let Ok(metadata) = entry.metadata() {
                            let size = metadata.len();
                            let colored_name = format_filename(&path, dir_color_rgb);
                            println!("{:>8}  {}", format_size(size), colored_name);
                        }
                    } else {
                        print!("{}  ", format_filename(&path, dir_color_rgb));
                    }
                }
            }

            if !show_long && !show_size {
                println!();
            }
        }
        Err(e) => eprintln!("Error reading directory: {}", e),
    }
}

fn print_tree(
    path: &Path,
    prefix: String,
    dir_color: Option<Color>,
    depth: usize,
    max_depth: Option<usize>,
    show_all: bool,
    pattern: Option<&Pattern>,
) {
    if max_depth.map_or(false, |m| depth > m) {
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => {
            let mut entries: Vec<_> = e.filter_map(Result::ok).collect();
            entries.sort_by_key(|e| e.file_name());
            entries
        }
        Err(_) => return,
    };

    let last = entries.len().saturating_sub(1);

    for (i, entry) in entries.into_iter().enumerate() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !show_all && file_name.starts_with('.') {
            continue;
        }

        let p = entry.path();

        if let Some(pat) = pattern {
            if !pat.matches_path(&p) {
                if p.is_dir() {
                    print_tree(
                        &p,
                        format!("{}{}", prefix, if i == last { "    " } else { "│   " }),
                        dir_color,
                        depth + 1,
                        max_depth,
                        show_all,
                        pattern,
                    );
                }
                continue;
            }
        }

        let is_last = i == last;
        let branch = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last { "    " } else { "│   " };

        let line = format_filename(&p, dir_color);
        println!("{}{}{}", prefix, branch, line);

        if p.is_dir() {
            print_tree(
                &p,
                format!("{}{}", prefix, new_prefix),
                dir_color,
                depth + 1,
                max_depth,
                show_all,
                pattern,
            );
        }
    }
}

fn format_permissions(mode: u32) -> String {
    let chars = [
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' },
    ];
    chars.iter().collect()
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::TrueColor { r, g, b })
}

fn normalize_hex_color(input: &str) -> String {
    let mut hex = input.trim_start_matches('#').to_uppercase();
    if hex.len() == 3 {
        hex = hex
            .chars()
            .map(|c| format!("{0}{0}", c))
            .collect::<Vec<_>>()
            .join("");
    }
    hex
}

fn format_filename(path: &Path, dir_color: Option<Color>) -> ColoredString {
    if path.is_dir() {
        let name = path.file_name().unwrap().to_string_lossy();
        if let Some(color) = dir_color {
            name.color(color).bold()
        } else {
            name.blue().bold()
        }
    } else if path.extension().map_or(false, |ext| ext == "rs") {
        path.file_name().unwrap().to_string_lossy().green()
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

fn get_config_path() -> Option<PathBuf> {
    home_dir().map(|mut p| {
        p.push(".peekconfig");
        p
    })
}

fn read_config(path: &Path) -> Option<Config> {
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write_config(path: &Path, config: &Config) -> std::io::Result<()> {
    let contents = serde_json::to_string_pretty(config).unwrap();
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())
}
