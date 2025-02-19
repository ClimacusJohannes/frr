use clap::Parser;
use find_and_replace::find_and_replace;
use iced::alignment::Horizontal::Left;
use iced::futures::never;
use iced::futures::stream::Collect;
use iced::widget::button::secondary;
use iced::widget::container::bordered_box;
use iced::widget::markdown::Url;
use iced::widget::text::{Rich, Span};
use iced::widget::text_editor::{Action, Content};
use iced::widget::{
    button, column, container, markdown, rich_text, row, scrollable, text, text_editor, text_input,
    Column, Container, Scrollable, Text,
};
use iced::{Element, Renderer, Size, Theme};
use rfd::FileDialog;
use std::fs::{self, DirEntry};
use std::io::{BufRead, BufReader, BufWriter, Write};

mod dir_crawl;
mod find_and_replace;

use dir_crawl::dir_crawl;

#[derive(Default, Clone)]
struct State {
    find: (String, String),
    replace: (String, String),
    path: String,
    text: String,
    markdown: Vec<markdown::Item>,
    confirm: bool,
    file_list: Vec<String>,
}

impl State {
    pub fn update_markdown(&mut self) {
        self.markdown = markdown::parse(&self.text).collect();
    }
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
    Nothing,
}

fn do_nothing(action: Url) -> Message {
    Message::Nothing
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
            container(scrollable(
                markdown::view(
                    &state.markdown,
                    markdown::Settings::default(),
                    markdown::Style::from_palette(Theme::SolarizedDark.palette()),
                )
                .map(do_nothing)
            ))
            .height(600)
            .width(10000)
            .padding(20)
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
            let mut temp_text = "".to_owned();
            if state.find.0 == "" || state.replace.0 == "" || state.path == "" {
                temp_text = "Please enter all three required parameters.".to_owned();
                state.text = format!("{}", &temp_text);

                return;
            } else if state.path.chars().next().unwrap() != '/' {
                temp_text = "Please enter an absolute path.".to_owned();
                state.text = format!("{}", &temp_text);

                return;
            }
            state.confirm = false;
            temp_text = format!(
                "Would you like to replace '{}' with '{}' in following files:\n\n --- \n\n",
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
                                    temp_text =
                                        format!("{}{}\n --- \n\n\n\n", &temp_text, text).clone();
                                }
                                state.confirm = true;
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                                temp_text = format!("{:?} There was a problem: {}", state.text, e);
                                state.confirm = false;
                            }
                        };
                    }
                    // set the content
                    state.text = format!("{}", &temp_text);
                    // update the file list
                    state.file_list = new_file_list;
                }
                Err(e) => eprintln!("There was a problem searching for txt files: {}", e),
            }
            state.update_markdown();
        }

        // event handling for the completion of the replace operation
        Message::Replace => {
            state.text = format!(
                "Success! Replaced '{}' with '{}' in the following files:\n\n",
                state.find.0, state.replace.0
            );
            state.find.1 = state.find.0.clone();
            state.replace.1 = state.replace.0.clone();
            for path in &state.file_list {
                match find_and_replace(&state.find.0, &state.replace.0, path) {
                    Ok(_) => state.text = format!("{}\n- '{}'\n", state.text, path),

                    Err(e) => {
                        state.text = format!("Error: {}", e);
                    }
                };
            }
            state.text = format!("{}", &state.text);
            state.confirm = false;
            state.update_markdown();
        }
        Message::Cancel => {
            state.confirm = false;
            state.text = format!("Operation cancelled.");
            state.update_markdown();
        }

        // event handling for the browse button
        Message::BrowsePath => {
            let path = FileDialog::new().pick_folder();
            match path {
                Some(path) => {
                    state.path = path.display().to_string();
                }
                None => {
                    state.text = format!("No path selected.");
                }
            }
            state.update_markdown();
        }
        Message::ChangePath(dir) => {
            state.path = dir;
            state.update_markdown();
        }

        // update path based on updated find or replace strings
        Message::UpdatePath(slice) => {
            let cloned_path = state.path.clone();
            if !cloned_path.contains(&slice.1) || slice.1 == "" {
                state.text =
                    format!("Could not update the path automatically, please update it manually.",);
                return;
            }
            let new_path = cloned_path.replace(&slice.1, &slice.0);
            state.path = new_path;
        }
        Message::Nothing => {}
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
            width: 5000.0,
            height: 1000.0,
        })
        .run()
}
