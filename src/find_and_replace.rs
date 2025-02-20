use anyhow::{bail, Context, Error, Result};
use log::kv::ToKey;
use std::{
    f32::consts::E,
    fmt::format,
    fs,
    io::{self, BufRead, BufWriter, Write},
};

use tokio::io::BufReader;

use iced::{
    color,
    widget::{text, Text},
};

// a function that will find all the occurrences of the pattern in a file (path)
// and display them visually

pub async fn find_from_vec(
    find_pat: String,
    replace: String,
    paths: Vec<String>,
) -> Result<String, Error> {
    let mut output = "".to_owned();

    for path in paths.into_iter() {
        let f = find(find_pat.to_owned(), replace.to_owned(), path.to_string()).await?;
        output = format!("{}{}", output, f);
    }

    if output == "" {
        bail!(Box::new("Nothing found"));
    } else {
        Ok(output)
    }
}

pub async fn find(find: String, replace: String, path: String) -> Result<String, Error> {
    let path = path.to_owned();
    let f = fs::File::open(&path)?;
    let reader = tokio::fs::read_to_string(&path).await?;
    let mut text = "".to_owned();
    let mut file_contains_pattern = false;

    for (num, line) in reader.lines().enumerate() {
        let mut unwraped_line = "".to_owned();

        unwraped_line = format!("{}", line);

        if unwraped_line.contains(&find) {
            file_contains_pattern = true;
            let display_line = display_line(&find, &replace, &unwraped_line, num + 1)
                .expect("Line was not able to be displayed.");
            text = format!("{}{}", text, display_line);
        }
    }

    if file_contains_pattern {
        text = format!("\n\n\n### File: '{}'\n\n\n{}", path, text);
    }

    Ok(text)
}

fn display_line(
    find: &str,
    replace: &str,
    line: &str,
    line_num: usize,
) -> Result<String, io::Error> {
    let mut output = format!("{}: \n", line_num);
    // caculate where to highlight the pattern
    let mut n = line.find(find).unwrap();
    let mut len = find.len();
    let m = line.len() - (n + len);
    let (old_line_1, old_line_temp) = line.clone().split_at(n);
    let (find, old_line_2) = old_line_temp.clone().split_at(len);
    let highlight_old_line = format!(
        "{}**[{}](https://en.wikipedia.org)**{}",
        old_line_1, find, old_line_2
    );
    // add the line with original pattern and a highlight line bellow

    let new_line = line.replace(find, replace).clone();
    n = new_line.find(replace).unwrap();
    len = replace.len();
    let binding = new_line.clone();
    let (new_line_1, new_line_temp) = binding.split_at(n);
    let (replace, new_line_2) = new_line_temp.clone().split_at(len);
    let highlight_new_line = format!(
        "{}**[{}](https://en.wikipedia.org)**{}",
        &new_line_1, replace, new_line_2
    );
    output = format!(
        "{}\n{}\n\n => {}\n\n",
        output, &highlight_old_line, &highlight_new_line
    );

    Ok(output)
}

pub async fn replace_from_vec(
    find_pat: String,
    replace: String,
    paths: Vec<String>,
) -> Result<String, Error> {
    let mut output = "".to_owned();

    for path in paths.into_iter() {
        let result =
            find_and_replace(find_pat.to_owned(), replace.to_owned(), path.to_string()).await;
        match result {
            Ok(_) => {
                output = format!("{}\n- '{}'\n", output, &path);
            }
            Err(e) => {
                // eprintln!("Error: {}", e);
            }
        }
    }

    Ok(output)
}

pub async fn find_and_replace(
    find: String,
    replace_with: String,
    path: String,
) -> Result<(), Error> {
    let f = fs::File::open(&path)?;
    let reader = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("Could not read file: '{}'", &path))?;
    let mut text = "".to_string();
    let mut file_contains_pattern = false;

    for (num, line) in reader.lines().enumerate() {
        let mut unwraped_line = "".to_owned();

        unwraped_line = format!("{}", line);

        if unwraped_line.contains(&find) {
            file_contains_pattern = true;
            let new_line = unwraped_line.replace(&find, &replace_with);
            text = format!("{}{}\n", &text, &new_line);
        } else {
            text = format!("{}{}\n", &text, &unwraped_line);
        }
    }

    let _ = tokio::fs::write(path.clone(), text)
        .await
        .with_context(|| format!("Error writing to file '{}'!", path));

    if file_contains_pattern {
        Ok(())
    } else {
        Err(Error::new(io::Error::new(
            io::ErrorKind::NotFound,
            "patter not found in file",
        )))
    }
}

mod tests {
    use serde::de::IntoDeserializer;

    use super::{find, find_and_replace};
    use std::{
        fs::{self, File},
        io::{self, Read},
    };

    fn create_file_with_contents(path: &str, contents: &str) -> Result<String, io::Error> {
        let _new_file = fs::File::create(path).unwrap();
        fs::write(path, contents).unwrap();
        Ok(path.to_owned())
    }

    fn bury_in_lorem_ipsum(contents: &str) -> String {
        format!("{}{}{}", lipsum::lipsum(100), contents, lipsum::lipsum(100))
    }

    #[test]
    fn find_and_replace_works_basic() {
        let path = ".test.txt";
        let find = "241220_LU02_tzajec_RNase_modifications_E0-0_01/241220_LU02_tzajec_RNase_modifications_E0-0_01.c.mzXML";
        let contents = bury_in_lorem_ipsum(find);
        let replace = "241220_LU02_tzajec_RNase_modifications_E0-0_02/241220_LU02_tzajec_RNase_modifications_E0-0_01.c.mzXML";
        let _ = create_file_with_contents(path, &contents).unwrap();
        let result = find_and_replace(find.to_owned(), replace.to_owned(), path.to_owned()).await;

        assert!(result.is_ok());

        // check if you can find the replaced pattern in the file
        let new_contents = fs::read_to_string(path).unwrap();

        assert!(new_contents.contains(replace));

        let remove = fs::remove_file(path);
        assert!(remove.is_ok())
    }

    #[test]
    fn find_works_basic() {
        let _ = create_file_with_contents(".text.txt", "izak");
        let result = find("izak", "tina", ".text.txt");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "File: '.text.txt'\n\n\n1: izak\n   ^^^^\n=> tina\n   ^^^^"
        );

        let remove = fs::remove_file(".text.txt");
        assert!(remove.is_ok())
    }

    #[test]
    fn find_and_replace_works_with_sample_data() {
        let path = "/home/izak/dev/tina/Rust/frr/data/test_files/241220_LU02_tzajec_RNase_modifications_E0-0_01/Search_1/parameters_search_1.txt";
        let result = find_and_replace(
            "241220_LU02_tzajec_RNase_modifications_E0-0_01",
            "241220_LU02_tzajec_RNase_modifications_E0-0_012345",
            path,
        );
        let mut file = fs::File::open(path).unwrap();
        let mut new_contents = "".to_owned();
        let read_result = file.read_to_string(&mut new_contents);
        assert!(result.is_ok());
        assert!(read_result.is_ok());
        assert!(result.is_ok());

        assert!(new_contents.contains("241220_LU02_tzajec_RNase_modifications_E0-0_012345"));

        let _ = find_and_replace(
            "241220_LU02_tzajec_RNase_modifications_E0-0_012345",
            "241220_LU02_tzajec_RNase_modifications_E0-0_01",
            path,
        );

        assert!(new_contents.contains("241220_LU02_tzajec_RNase_modifications_E0-0_01"));
    }

    #[test]
    fn find_finds_the_pattern() {
        let path = "/home/izak/dev/tina/Rust/frr/data/test_files/241220_LU02_tzajec_RNase_modifications_E0-0_01/Search_2/parameters_search_2.txt";
        let result = find(
            "241220_LU02_tzajec_RNase_modifications_E0-0_01",
            "241220_LU02_tzajec_RNase_modifications_E0-0_012345",
            path,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("^^^^"));
    }
}
