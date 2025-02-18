use anyhow::{Context, Error, Result};
use log::kv::ToKey;
use std::{
    fmt::format,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

use iced::{
    color,
    widget::{text, Text},
};

// a function that will find all the occurrences of the pattern in a file (path)
// and display them visually
pub fn find(find: &str, replace: &str, path: &str) -> Result<String, Error> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    let mut text = "".to_owned();
    let mut file_contains_pattern = false;

    for (num, line) in reader.lines().enumerate() {
        let mut unwraped_line = "".to_owned();

        match line.with_context(|| format!("Failed to read line {}", num + 1)) {
            Ok(line) => {
                unwraped_line = line;
            }
            Err(e) => {
                eprintln!("While reading file '{}' and error occurred: {}", path, e);
            }
        }

        if unwraped_line.contains(find) {
            file_contains_pattern = true;
            let display_line = display_line(find, replace, &unwraped_line, num + 1)
                .expect("Line was not able to be displayed.");
            text = format!("{}{}", text, display_line);
        }
    }

    if file_contains_pattern {
        text = format!("File: '{}'\n\n{}", path, text);
    }

    Ok(text)
}

fn display_line(
    find: &str,
    replace: &str,
    line: &str,
    line_num: usize,
) -> Result<String, io::Error> {
    let mut output = "".to_owned();
    // caculate where to highlight the pattern
    let n = line.find(find).unwrap();
    let mut len = find.len();
    let m = line.len() - (n + len);
    let highlight_line = format!(
        "{}{}",
        (0..n).map(|_| " ").collect::<Vec<_>>().concat(),
        (0..len).map(|_| "-").collect::<Vec<_>>().concat()
    );
    // add the line with original pattern and a highlight line bellow
    output = format!("{}\n{}: {}\n   {}", output, line_num, line, highlight_line);

    let new_line = line.replace(find, replace);
    len = replace.len();
    let replaced_highlight_line = format!(
        "{}{}",
        (0..n).map(|_| " ").collect::<Vec<_>>().concat(),
        (0..len).map(|_| "+").collect::<Vec<_>>().concat()
    );
    output = format!(
        "{}\n=> {}\n   {}",
        output, new_line, replaced_highlight_line
    );

    Ok(output)
}

pub fn find_and_replace(find: &str, replace_with: &str, path: &str) -> Result<(), Error> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    let mut text = "".to_string();

    for (num, line) in reader.lines().enumerate() {
        let mut unwraped_line = "".to_owned();

        match line.with_context(|| format!("Failed to read line {}", num + 1)) {
            Ok(line) => {
                unwraped_line = line;
            }
            Err(e) => {
                eprintln!(
                    "While replacing the pattern in file '{}' and error occured: {}",
                    path, e
                );
                continue;
            }
        }

        if unwraped_line.contains(find) {
            let new_line = unwraped_line.replace(find, replace_with);
            text = format!("{}{}\n", &text, &new_line);
        } else {
            text = format!("{}{}\n", &text, &unwraped_line);
        }
    }

    // stripping the last line
    // text = text
    //     .strip_suffix("\n")
    //     .expect("Count not strip suffix")
    //     .to_owned();

    // writing to file
    let _ = fs::write(path, text).with_context(|| format!("Error writing to file '{}'!", path));

    Ok(())
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
        let result = find_and_replace(find, replace, path);

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
