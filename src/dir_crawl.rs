use anyhow::Error;
use anyhow::{Context, Result};
use std::fs;

pub fn dir_crawl(org_path: &str) -> Result<Vec<String>, Error> {
    let paths = fs::read_dir(org_path)?;

    let mut list: Vec<String> = vec![];

    for path in paths {
        let path = path?;
        let is_dir = &path.path().is_dir();

        // if a directory, recursively call find_and_replace
        if is_dir.to_owned() {
            for path in dir_crawl(&path.path().display().to_string())
                .with_context(|| format!("Could not enter {:?}", &path))?
            {
                list.push(path.clone());
            }
        } else if path.path().display().to_string().ends_with(".txt") {
            let can_path = fs::canonicalize(path.path().display().to_string())?
                .display()
                .to_string();
            list.push(can_path);
        }
    }
    Ok(list)
}

mod tests {
    use std::fs;

    use super::dir_crawl;

    #[test]
    fn dir_crawl_is_ok() {
        let result = dir_crawl(".");

        assert!(result.is_ok());
    }

    #[test]
    fn dir_crawl_finds_created_file() {
        let create = fs::File::create(".dir_crawl.txt");
        assert!(create.is_ok());

        let result = dir_crawl(".").unwrap();
        assert!(result.contains(&"/home/izak/dev/tina/Rust/frr/.dir_crawl.txt".to_owned()));

        let remove = fs::remove_file(".dir_crawl.txt");
        assert!(remove.is_ok())
    }
}
