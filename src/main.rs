#[macro_use]
extern crate clap;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

#[derive(Debug, Clone)]
pub struct Config {
    pub dry_run: bool,
    pub verbose: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            dry_run: false,
            verbose: false,
        }
    }
}

fn remove_file(p: &Path, config: &Config) {
    if config.dry_run {
        println!("To delete: {:?}", p);
    } else {
        if config.verbose {
            println!("Deleting {:?}", p);
        }
        if let Err(e)= fs::remove_file(p) {
            eprintln!("{}", e);
        }
    }
}

fn remove_dir(p: &Path, config: &Config) {
    if config.dry_run {
        println!("To delete: {:?}", p);
    } else {
        if config.verbose {
            println!("To delete: {:?}", p);
        }
        if let Err(e) = fs::remove_dir_all(p) {
            eprintln!("{}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn handle(config: &Config) {

    let app_data_str = env::var("AppData");
    if let Err(e) = app_data_str {
        eprintln!("Unable to obtain environment variable: AppData");
        eprintln!("{}", e);
        return;
    }


    let app_data_str = app_data_str.unwrap();
    let app_data = Path::new(app_data_str.as_str());
    if config.verbose {
        println!("AppData: {:?}", app_data);
    }
    let p_teams = app_data.join("Microsoft").join("Teams");
    if !p_teams.is_dir() {
        eprintln!("Teams was not found in {:?}", p_teams);
        return;
    }
    let p_service_worker = p_teams.join("Service Worker");
    let dirs_to_clean = vec![p_teams];
    let list = vec![vec![
        "blob_storage",
        "databases",
        "GPUCache",
        "IndexedDB",
        "Local Storage",
        "tmp",
        "Cache",
    ]];
    for (dir, vsub) in dirs_to_clean.iter().zip(list.iter()) {
        if !dir.is_dir() {
            continue;
        }
        for sub in vsub {
            let sub_dir = dir.join(*sub);
            if !sub_dir.is_dir() {
                continue;
            }
            let paths = fs::read_dir(sub_dir);
            if let Err(e) = paths {
                eprintln!("{}", e);
                exit(1);
            }

            let paths = paths.unwrap();
            for res_entry in paths {
                if let Err(e) = res_entry {
                    eprintln!("{}", e);
                    exit(1);
                }
                let entry = res_entry.unwrap();
                match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_file() {
                            remove_file(&entry.path(), &config);
                            // println!("To delete: {:?}", entry.path());
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(1);
                    }
                }
            }
        }
    }

    let vcache = vec![
        p_service_worker.join("CacheStorage"),
        p_service_worker.join("ScriptCache"),
    ];
    for cache in &vcache {
        if !cache.is_dir() {
            continue;
        }
        for res_paths in fs::read_dir(cache) {
            for entry in res_paths {
                if let Err(e) = entry {
                    eprintln!("{}", e);
                    exit(1);
                }
                let entry = entry.unwrap();
                let file_type = entry.file_type().unwrap();
                if file_type.is_file() {
                    remove_file(&entry.path(), &config);
                    // println!("To delete: {:?}", entry.path());
                    // fs::remove_file(entry.path());
                } else if file_type.is_dir() {
                    remove_dir(&entry.path(), &config);
                    // println!("To delete: {:?}", entry.path());
                    // fs::remove_dir_all(entry_path());
                } else {
                    eprintln!("Unsupported file type: {:?}", file_type);
                    exit(1);
                }
            }
        }
    }

}

fn main() {
    let mut config = Config::default();
    let matches = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("dry-run")
                .help("Print a list of files that application wants to delete.")
                .takes_value(false)
                .short("d")
                .long("dry-run")
                .multiple(false)
                .required(false),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Print which files or directories are being removed.")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .multiple(false)
                .required(false),
        )
        .get_matches();

    if matches.is_present("dry-run") {
        config.dry_run = true;
        // println!("Dry-run enabled ...");
    }
    if matches.is_present("verbose") {
        config.verbose = true;
        // println!("Verbose enabled ...");
    }

    handle(&config);
}
