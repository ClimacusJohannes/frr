use clap::Parser;
use find_and_replace::{find, find_and_replace, find_from_vec, replace_from_vec};
use iced::alignment::Horizontal::Left;
use iced::border::Radius;
use iced::futures::never;
use iced::futures::stream::Collect;
use iced::widget::button::secondary;
use iced::widget::container::{bordered_box, rounded_box};
use iced::widget::markdown::Url;
use iced::widget::scrollable::{scroll_by, AbsoluteOffset, Id};
use iced::widget::shader::wgpu::hal::TextureFormatCapabilities;
use iced::widget::text::{Rich, Span};
use iced::widget::text_editor::{Action, Content};
use iced::widget::{
    button, column, container, markdown, rich_text, row, scrollable, text, text_editor, text_input,
    Button, Column, Container, Scrollable, Text,
};
use iced::{keyboard, Border, Element, Renderer, Size, Task, Theme};
use log::kv::ToKey;
use rfd::{AsyncFileDialog, FileDialog};
use std::fmt::format;
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
    focus: String,
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
    EnableConfirm(String),
    Confirm(String),
    AddText(String),
    Replace,
    Cancel,
    Nothing,
    EnterKeyPressed,
    TabKeyPressed,
    MoveUp,
    MoveDown,
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
                        .id("find")
                        .on_input_maybe(if !state.confirm {
                            Some(Message::FindChanged)
                        } else {
                            Option::None
                        })
                        .on_submit(Message::EnterKeyPressed),
                    text_input("Replace with", &state.replace.0)
                        .id("replace")
                        .on_input_maybe(if !state.confirm {
                            Some(Message::ReplaceChanged)
                        } else {
                            Option::None
                        })
                        .on_submit(Message::EnterKeyPressed),
                    row![
                        text_input("Directory", &state.path)
                            .id("dir")
                            .on_input_maybe(if !state.confirm {
                                Some(Message::ChangePath)
                            } else {
                                Option::None
                            })
                            .on_submit(Message::EnterKeyPressed),
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
                        .on_press_maybe(if !state.confirm {
                            Some(Message::UpdatePath(state.find.clone()))
                        } else {
                            Option::None
                        })
                        .width(220)
                        .height(35)
                        .style(secondary),
                    button("Update path - replace")
                        .on_press_maybe(if !state.confirm {
                            Some(Message::UpdatePath(state.replace.clone()))
                        } else {
                            Option::None
                        })
                        .width(220)
                        .height(35)
                        .style(secondary),
                    button("Find")
                        .on_press_maybe(if !state.confirm {
                            Some(Message::Find)
                        } else {
                            Option::None
                        })
                        .width(220)
                        .height(35),
                ]
                .spacing(20),
            ]
            .spacing(20),
            // scrollable text output
            container(
                scrollable(
                    markdown::view(
                        &state.markdown,
                        markdown::Settings::default(),
                        markdown::Style::from_palette(Theme::CatppuccinLatte.palette()),
                    )
                    .map(do_nothing)
                )
                .id(Id::new("scrollable"))
            )
            .height(600)
            .width(10000)
            .padding(20)
            .style(rounded_box),
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

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // event handling for the find text input
        Message::FindChanged(find) => {
            let mut trunc_find = find.clone();
            let len = find.trim_end_matches(&['\r', '\n'][..]).len();
            trunc_find.truncate(len);
            state.find.0 = trunc_find;
            Task::none()
        }

        // event handling for the replace text input
        Message::ReplaceChanged(replace) => {
            let mut trunc_replace = replace.clone();
            let len = replace.trim_end_matches(&['\r', '\n'][..]).len();
            trunc_replace.truncate(len);
            state.replace.0 = trunc_replace;
            Task::none()
        }

        // event handling for the provisional replace results
        Message::Find => {
            state.confirm = false;
            if state.find.0 == "" || state.replace.0 == "" || state.path == "" {
                return Task::done(Message::AddText(
                    "Please enter all three required parameters.".to_owned(),
                ));
            } else if state.path.chars().next().unwrap() != '/' {
                return Task::done(Message::AddText(
                    "Please enter an absolute path.".to_owned(),
                ));
            }
            state.text = "# Searching...".to_owned();
            state.update_markdown();
            match dir_crawl(&state.path) {
                Ok(list) => {
                    state.file_list = list.clone();
                    let mut new_file_list: Vec<String> = vec![];
                    return Task::perform(
                        find_from_vec(
                            state.find.0.to_owned(),
                            state.replace.0.to_owned(),
                            state.file_list.clone(),
                        ),
                        |text| match text {
                            Ok(text) => {
                                return Message::EnableConfirm(text);
                            }
                            Err(e) => return Message::AddText(format!("{}", e)),
                        },
                    );
                }
                Err(e) => {
                    eprintln!("There was a problem searching for txt files: {}", e);
                }
            }
            state.update_markdown();
            Task::none()
        }

        Message::EnableConfirm(text) => {
            state.confirm = true;
            Task::done(Message::AddText(text))
        }

        Message::Confirm(text) => {
            state.confirm = false;
            Task::done(Message::AddText(format!(
                "Replaced '{}' with '{}' in the following files: {}",
                state.find.0, state.replace.0, text
            )))
        }

        Message::AddText(text) => {
            state.text = format!("{}", text);
            state.update_markdown();
            Task::none()
        }

        // event handling for the completion of the replace operation
        Message::Replace => {
            state.text = "# Replacing...".to_owned();
            state.update_markdown();
            // saving the find and replace for the path formatting
            state.find.1 = state.find.0.clone();
            state.replace.1 = state.replace.0.clone();

            state.confirm = false;

            return Task::perform(
                replace_from_vec(
                    state.find.0.to_owned(),
                    state.replace.0.to_owned(),
                    state.file_list.clone(),
                ),
                |text| match text {
                    Ok(text) => return Message::Confirm(text),
                    Err(e) => Message::Confirm(format!("{}", e)),
                },
            );
        }

        Message::Cancel => {
            state.confirm = false;
            state.text = format!("Operation cancelled.");
            state.update_markdown();
            Task::none()
        }

        // event handling for the browse button
        Message::BrowsePath => {
            return Task::perform(AsyncFileDialog::new().pick_folder(), |path| {
                Message::ChangePath(path.unwrap().path().display().to_string())
            });
        }
        Message::ChangePath(dir) => {
            state.path = dir;
            state.update_markdown();
            Task::none()
        }

        // update path based on updated find or replace strings
        Message::UpdatePath(slice) => {
            let cloned_path = state.path.clone();
            if !cloned_path.contains(&slice.1) || slice.1 == "" {
                state.text =
                    format!("Could not update the path automatically, please update it manually.",);
            }
            state.update_markdown();
            let new_path = cloned_path.replace(&slice.1, &slice.0);
            state.path = new_path;
            Task::none()
        }

        Message::EnterKeyPressed => {
            if state.confirm {
                return Task::done(Message::Replace);
            }
            Task::done(Message::Find)
        }

        Message::TabKeyPressed => {
            let ids = vec!["find", "replace", "dir"];
            let mut ids_iter = ids.clone().into_iter();
            loop {
                match ids_iter.next() {
                    Some(id) => {
                        if id == state.focus {
                            state.focus = ids_iter
                                .next()
                                .or_else(|| Some(ids.get(0).unwrap()))
                                .unwrap()
                                .to_string();
                            break;
                        }
                    }
                    None => {
                        state.focus = ids.get(0).unwrap().to_string();
                        break;
                    }
                }
            }

            text_input::focus(state.focus.as_str().to_owned())
        }

        Message::MoveDown => scroll_by(
            Id::new("scrollable"),
            AbsoluteOffset {
                x: 0.0 as f32,
                y: 15.0 as f32,
            },
        ),

        Message::MoveUp => scroll_by(
            Id::new("scrollable"),
            AbsoluteOffset {
                x: 0.0 as f32,
                y: -15.0 as f32,
            },
        ),

        Message::Nothing => Task::none(),
    }
}

fn subscription(_state: &State) -> iced::Subscription<Message> {
    fn handle_hotkey(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
        match key {
            keyboard::key::Key::Named(keyboard::key::Named::Enter) => {
                Some(Message::EnterKeyPressed)
            }
            keyboard::key::Key::Named(keyboard::key::Named::Tab) => Some(Message::TabKeyPressed),
            // keyboard::Key::Character("n") => Some(Message::OtherMessageToCall),
            keyboard::key::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::MoveUp),
            keyboard::key::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::MoveDown),
            _ => None,
        }
    }

    keyboard::on_key_press(handle_hotkey)
}

fn main() -> iced::Result {
    // let args = Args::parse();

    // find_and_replace(&args.find, &args.replace_with, &args.directory).unwrap();
    iced::application("Recursive find and replace for .txt files", update, view)
        .subscription(subscription)
        .theme(|_| Theme::CatppuccinLatte)
        .window_size(Size {
            width: 5000.0,
            height: 1000.0,
        })
        .run()
}
