use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Default, Debug)]
#[command(
    version,
    about = "Traverse the directory specified by $FP_FOLDER or $GOPATH to find a folder depth-first."
)]
struct Args {
    #[clap(required = true)]
    folder_name: String,

    #[clap(long, help = "Also search in \"vendor\" folders")]
    include_vendor: bool,

    #[clap(long, help = "Also search in hidden (dot) folders")]
    include_hidden: bool,

    #[clap(long, help = "Sort folders alphabetically")]
    sort_alphabetically: bool,
}

fn main() -> Result<()> {
    // Collect command-line arguments
    let args = Args::parse();

    // Enable debug logging if the environment variable FP_DEBUG is set
    // to any non-empty value.
    let log_enabled = env::var("FP_DEBUG").is_ok();

    // Check if FP_FOLDER or GOPATH are set:
    // If FP_FOLDER is set, use it as the folder to search,
    // otherwise, use the GOPATH folder.
    let location = env::var("GOPATH")
        .or_else(|_| env::var("FP_FOLDER"))
        .context(
            "Please set the $FP_FOLDER environment variable or the $GOPATH \
            environment variable to a location that find-project can search. \
            Neither are set.",
        )?;

    // If the path is a $GOPATH, then append "src" to it, otherwise,
    // use the path as is
    let full_location = if env::var("GOPATH").is_ok() {
        Path::new(&location)
            .join("src")
            .canonicalize()
            .context("Unable to get absolute path to $GOPATH/src")?
    } else {
        Path::new(&location)
            .to_path_buf()
            .canonicalize()
            .context("Unable to get absolute path to $FP_FOLDER")?
    };

    // Find the directory
    let loc = finddir(&full_location, &args, log_enabled)?;
    if let Some(loc) = loc {
        println!("{}", loc.display());
        Ok(())
    } else {
        eprintln!(
            "Folder \"{}\" not found inside {}",
            args.folder_name,
            full_location.as_os_str().to_string_lossy()
        );
        std::process::exit(1);
    }
}

fn finddir(p: &Path, args: &Args, log_enabled: bool) -> Result<Option<PathBuf>> {
    let mut dirs = getalldirs(p, args)?;
    let name = Path::new(&args.folder_name);

    let mut i = 0;
    while i < dirs.len() {
        let dir = &dirs[i];
        if log_enabled {
            eprintln!("Searching in: {}", dir.display());
        }

        if let Some(base_name) = dir.file_name() {
            if base_name == name {
                return Ok(Some(dir.clone()));
            }
        }

        let extras = getalldirs(dir, args)?;
        for extra in extras {
            if let Some(extra_name) = extra.file_name() {
                if extra_name == name {
                    if log_enabled {
                        eprintln!("Found: {}", extra.display());
                    }
                    return Ok(Some(extra));
                }
            }
            dirs.push(extra);
        }

        i += 1;
    }

    Ok(None)
}

fn getalldirs(p: &Path, args: &Args) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    let entries = fs::read_dir(p).with_context(|| format!("Unable to read directory {:?}", p))?;
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if !args.include_hidden && name_str.starts_with('.') {
                continue;
            }

            if !args.include_vendor && name_str == "vendor" {
                continue;
            }

            dirs.push(entry.path());
        }
    }

    if args.sort_alphabetically {
        dirs.sort();
    }

    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_getalldirs() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();

        let args = Args {
            include_vendor: false,
            include_hidden: false,
            ..Default::default()
        };

        // Create some directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("vendor"))?;
        fs::create_dir(path.join(".hidden"))?;
        fs::create_dir(path.join("dir1").join("subdir1"))?;

        let dirs = getalldirs(path, &args)?;

        assert_eq!(dirs.len(), 2);
        let mut names = dirs
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, vec!["dir1", "dir2"]);

        Ok(())
    }

    #[test]
    fn test_finddir() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: false,
            include_hidden: false,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("target"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join("dir2").join("vendor").join("target"))?;
        fs::create_dir(path.join(".hidden"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_some());
        let found_path = found.unwrap();
        assert_eq!(found_path, path.join("dir1").join("target"));

        Ok(())
    }

    #[test]
    fn test_finddir_not_found() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: false,
            include_hidden: false,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("subdir1"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join(".hidden"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_none());

        Ok(())
    }

    #[test]
    fn test_finddir_inside_vendor() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: true,
            include_hidden: false,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("extra"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join("dir2").join("vendor").join("target"))?;
        fs::create_dir(path.join(".hidden"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_some());
        let found_path = found.unwrap();
        assert_eq!(found_path, path.join("dir2").join("vendor").join("target"));

        Ok(())
    }

    #[test]
    fn test_find_inside_vendor_disabled() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: false,
            include_hidden: false,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("extra"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join("dir2").join("vendor").join("target"))?;
        fs::create_dir(path.join(".hidden"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_none());

        Ok(())
    }

    #[test]
    fn test_find_inside_hidden() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: false,
            include_hidden: true,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("extra"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join("dir2").join("vendor").join("target"))?;
        fs::create_dir(path.join(".hidden"))?;
        fs::create_dir(path.join(".hidden").join("target"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_some());
        let found_path = found.unwrap();
        assert_eq!(found_path, path.join(".hidden").join("target"));

        Ok(())
    }

    #[test]
    fn test_find_inside_hidden_disabled() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        let args = Args {
            include_vendor: false,
            include_hidden: false,
            sort_alphabetically: false,
            folder_name: "target".to_string(),
        };

        // Create directories
        fs::create_dir(path.join("dir1"))?;
        fs::create_dir(path.join("dir2"))?;
        fs::create_dir(path.join("dir1").join("extra"))?;
        fs::create_dir(path.join("dir2").join("vendor"))?;
        fs::create_dir(path.join("dir2").join("vendor").join("target"))?;
        fs::create_dir(path.join(".hidden"))?;
        fs::create_dir(path.join(".hidden").join("target"))?;

        let found = finddir(path, &args, false)?;
        assert!(found.is_none());

        Ok(())
    }
}
