use clap::Parser;
use find_and_replace::find_and_replace;
use iced::alignment::Horizontal::Left;
use iced::widget::button::secondary;
use iced::widget::container::bordered_box;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, Column, Container,
};
use iced::Length::Fill;
use iced::Size;
use log::info;
use rfd::FileDialog;
use std::fs::{self, DirEntry};
use std::io::{BufRead, BufReader, BufWriter, Write};

mod dir_crawl;
mod find_and_replace;

use dir_crawl::dir_crawl;

#[derive(Default)]
struct State {
    find: (String, String),
    replace: (String, String),
    path: String,
    text: String,
    confirm: bool,
    file_list: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    FindChanged(String),
    ReplaceChanged(String),
    BrowsePath,
    UpdatePath((String, String)),
    ChangePath(String),
    Find,
    Replace,
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
                            .on_press_maybe(if !state.confirm {
                                Some(Message::BrowsePath)
                            } else {
                                Option::None
                            })
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
                    button("Find").on_press(Message::Find).width(220).height(35),
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
                button("Replace")
                    .on_press_maybe(if state.confirm {
                        Some(Message::Replace)
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
        Message::Find => {
            if state.find.0 == "" || state.replace.0 == "" || state.path == "" {
                state.text = "Please enter all three required parameters.".to_string();
                return;
            } else if state.path.chars().next().unwrap() != '/' {
                state.text = "Please enter an absolute path.".to_string();
                return;
            }
            state.confirm = false;
            state.text = format!(
                "Would you like to replace '{}' with '{}' in following files:\n\n-------------------------------------------------------------------\n\n",
                    state.find.0, state.replace.0
            );
            match dir_crawl(&state.path) {
                Ok(list) => {
                    state.file_list = list.clone();
                    let mut new_file_list: Vec<String> = vec![];
                    for path in list {
                        match find_and_replace::find(&state.find.0, &state.replace.0, &path) {
                            Ok(text) => {
                                if text != "" {
                                    // add the path to the updated file list
                                    new_file_list.push(path);
                                    state.text = format!("{}{}\n\n", state.text, text);
                                    state.text = format!("{}---------------------------------------------------------------------------------\n\n", state.text);
                                }
                                state.confirm = true;
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                                state.text = format!("{} There was a problem: {}", state.text, e);
                                state.confirm = false;
                            }
                        };
                    }
                    // update the file list
                    state.file_list = new_file_list;
                }
                Err(e) => eprintln!("There was a problem searching for txt files: {}", e),
            }
        }

        // event handling for the completion of the replace operation
        Message::Replace => {
            state.text = format!(
                "Success! Replaced '{}' with '{}' in the following files:\n\n",
                state.find.0, state.replace.0
            )
            .to_string();
            state.find.1 = state.find.0.clone();
            state.replace.1 = state.replace.0.clone();
            for path in &state.file_list {
                match find_and_replace(&state.find.0, &state.replace.0, path) {
                    Ok(_) => state.text = format!("{}\n'{}'\n", state.text, path),

                    Err(e) => {
                        eprintln!("{:?}", e);
                        state.text = format!("Error: {}", e);
                    }
                };
            }
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
