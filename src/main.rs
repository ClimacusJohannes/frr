use clap::Parser;
use iced::alignment::Horizontal::Left;
use iced::widget::button::secondary;
use iced::widget::container::bordered_box;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, Column, Container,
};
use iced::Length::Fill;
use iced::Size;
use rfd::FileDialog;
use std::fs;

#[derive(Default)]
struct State {
    find: (String, String),
    replace: (String, String),
    path: String,
    text: String,
    confirm: bool,
}

#[derive(Debug, Clone)]
enum Message {
    FindChanged(String),
    ReplaceChanged(String),
    BrowsePath,
    UpdatePath((String, String)),
    ChangePath(String),
    Replace,
    Confirm,
    Cancel,
}

fn view(state: &State) -> Container<'_, Message> {
    container(
        column![
            row![
                // text input for find, replace and directory
                column![
                    text_input("Find", &state.find.0)
                        .on_input_maybe(if !state.confirm {
                            Some(Message::FindChanged)
                        } else {
                            Option::None
                        })
                        .on_submit(Message::Replace),
                    text_input("Replace with", &state.replace.0)
                        .on_input_maybe(if !state.confirm {
                            Some(Message::ReplaceChanged)
                        } else {
                            Option::None
                        })
                        .on_submit(Message::Replace),
                    row![
                        text_input("Directory", &state.path)
                            .on_input_maybe(if !state.confirm {
                                Some(Message::ChangePath)
                            } else {
                                Option::None
                            })
                            .on_submit(Message::Replace),
                        button("Browse")
                            .on_press(Message::BrowsePath)
                            .width(80)
                            .height(35),
                    ]
                    .spacing(10),
                ]
                .max_width(500)
                .spacing(20),
                //buttons for updating path and running the find and replace operation
                column![
                    button("Update path - find (info)")
                        .on_press(Message::UpdatePath(state.find.clone()))
                        .width(220)
                        .height(35)
                        .style(secondary),
                    button("Update path - replace")
                        .on_press(Message::UpdatePath(state.replace.clone()))
                        .width(220)
                        .height(35)
                        .style(secondary),
                    button("Find and replace")
                        .on_press(Message::Replace)
                        .width(220)
                        .height(35),
                ]
                .spacing(20),
            ]
            .spacing(20),
            // scrollable text output
            container(
                column![
                    scrollable(text!("{}", &state.text).font(iced::Font::MONOSPACE))
                        .height(600)
                        .width(10000)
                ]
                .padding(20)
            )
            .style(bordered_box),
            // buttons for confirming or cancelling the operation
            row![
                button("Confirm")
                    .on_press_maybe(if state.confirm {
                        Some(Message::Confirm)
                    } else {
                        Option::None
                    })
                    .height(100),
                button("Cancel")
                    .on_press_maybe(if state.confirm {
                        Some(Message::Cancel)
                    } else {
                        Option::None
                    })
                    .height(100),
            ]
            .height(40)
            .spacing(20)
        ]
        .spacing(15)
        .padding(20),
    )
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        // event handling for the find text input
        Message::FindChanged(find) => {
            let mut trunc_find = find.clone();
            let len = find.trim_end_matches(&['\r', '\n'][..]).len();
            trunc_find.truncate(len);
            state.find.0 = trunc_find;
        }

        // event handling for the replace text input
        Message::ReplaceChanged(replace) => {
            let mut trunc_replace = replace.clone();
            let len = replace.trim_end_matches(&['\r', '\n'][..]).len();
            trunc_replace.truncate(len);
            state.replace.0 = trunc_replace;
        }

        // event handling for the provisional replace results
        Message::Replace => {
            if state.find.0 == "" || state.replace.0 == "" || state.path == "" {
                state.text = "Please enter all three required parameters.".to_string();
                return;
            } else if state.path.chars().next().unwrap() != '/' {
                state.text = "Please enter an absolute path.".to_string();
                return;
            }
            state.confirm = false;
            state.text = format!(
                "Would you like to replace '{}' with '{}' in following files:\n\n-------------------------------------------------------------------",
                    state.find.0, state.replace.0
            );
            match find_and_replace(
                &state.find.0,
                &state.replace.0,
                &state.path,
                &mut state.text,
                state.confirm,
            ) {
                Ok(text) => {
                    state.text = format!("{}\n\nDo you wish to continue?", text);
                    state.confirm = true;
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    state.text = format!("Error: '{}' - {}", &state.path, e);
                    state.confirm = true;
                }
            };
        }

        // event handling for the completion of the replace operation
        Message::Confirm => {
            state.text = format!(
                "Success! Replaced '{}' with '{}' in the following files:\n\n-------------------------------------------------------------------",
                state.find.0, state.replace.0
            )
            .to_string();
            state.find.1 = state.find.0.clone();
            state.replace.1 = state.replace.0.clone();
            match find_and_replace(
                &state.find.0,
                &state.replace.0,
                &state.path,
                &mut state.text,
                state.confirm,
            ) {
                Ok(text) => state.text = text,
                Err(e) => {
                    eprintln!("{:?}", e);
                    state.text = format!("Error: '{}' - {}", &state.path, e);
                }
            };
            state.confirm = false;
        }
        Message::Cancel => {
            state.confirm = false;
            state.text = "Operation cancelled.".to_string();
        }

        // event handling for the browse button
        Message::BrowsePath => {
            let path = FileDialog::new().pick_folder();
            match path {
                Some(path) => {
                    state.path = path.display().to_string();
                }
                None => {
                    state.text = "No path selected.".to_string();
                }
            }
        }
        Message::ChangePath(dir) => {
            state.path = dir;
        }

        // update path based on updated find or replace strings
        Message::UpdatePath(slice) => {
            let cloned_path = state.path.clone();
            if !cloned_path.contains(&slice.1) || slice.1 == "" {
                state.text = "Could not update the path automatically, please update it manually."
                    .to_string();
                return;
            }
            let new_path = cloned_path.replace(&slice.1, &slice.0);
            state.path = new_path;
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The string to find
    #[arg(short, long)]
    find: String,

    /// what to replace the find string with
    #[arg(short, long)]
    replace_with: String,

    /// Directory to search for files
    #[arg(short, long, default_value = "data")]
    directory: String,
}

fn main() -> iced::Result {
    // let args = Args::parse();

    // find_and_replace(&args.find, &args.replace_with, &args.directory).unwrap();
    iced::application("Recursive find and replace for .txt files", update, view)
        .window_size(Size {
            width: 1000.0,
            height: 1000.0,
        })
        .run()
}

fn find_and_replace(
    find: &str,
    replace_with: &str,
    org_path: &str,
    text: &mut String,
    confirm: bool,
) -> Result<String, std::io::Error> {
    let paths = fs::read_dir(org_path)?;
    let output_text = text;

    for path in paths {
        let path = path?;
        let is_dir = &path.path().is_dir();

        // if a directory, recursively call find_and_replace
        if is_dir.to_owned() {
            find_and_replace(
                find,
                replace_with,
                &path.path().display().to_string(),
                output_text,
                confirm,
            )?;
        } else if path.path().display().to_string().contains(".txt") {
            // add display_path to file in the output text
            let display_path = &path.path().display().to_string();

            // read file
            let buffer = fs::read_to_string(&path.path())?;

            if !buffer.contains(find) {
                if !confirm {
                    *output_text = format!("{}\n\nFile: {}\n\n", output_text, display_path);

                    *output_text =
                        format!("{}'{}' not found in file.\n\n", output_text, find).to_owned();
                }
                continue;
            }

            // change a string slice in the buffer
            let new_buffer = buffer.replace(find, replace_with);

            *output_text = format!("{}\n\nFile: {}\n\n", output_text, display_path);
            // display lines with the replaced slice in the buffer
            for (i, line) in buffer.lines().enumerate() {
                if line.contains(find) {
                    *output_text = format!("{}{}: {}\n", output_text, i + 1, line);
                    *output_text = format!(
                        "{}=> {}\n\n",
                        output_text,
                        new_buffer.lines().nth(i).unwrap()
                    );
                }
            }

            *output_text = format!(
                "{}-------------------------------------------------------------------\n",
                output_text
            );

            if confirm {
                fs::write(&path.path(), new_buffer)?;
            }
        } else {
            // in case of a non-txt file skip
            continue;
        }
    }

    Ok(output_text.to_owned())
}
