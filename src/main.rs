use find_and_replace::{find_from_vec, replace_from_vec};
use iced::widget::button::Status;
use iced::widget::markdown::Url;
use iced::widget::scrollable::{scroll_by, AbsoluteOffset, Id};
use iced::widget::{button, column, container, markdown, row, scrollable, text_input, Container};
use iced::{keyboard, Size, Task, Theme};
use rfd::AsyncFileDialog;

mod dir_crawl;
mod find_and_replace;
mod has_border;

use dir_crawl::dir_crawl;
use has_border::HasBorder;

#[derive(Clone)]
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

impl Default for State {
    fn default() -> Self {
        State {
            find: ("".to_owned(), "".to_owned()),
            replace: ("".to_owned(), "".to_owned()),
            path: "".to_owned(),
            text: "".to_owned(),
            markdown: markdown::parse("").collect(),
            confirm: false,
            file_list: vec!["".to_owned()],
            focus: "find".to_owned(),
        }
    }
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

fn do_nothing(_action: Url) -> Message {
    Message::Nothing
}

const BORDER_RADIUS: f32 = 7.5;

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
                        .style(|theme, status: text_input::Status| {
                            text_input::default(theme, status).set_border_radius(BORDER_RADIUS)
                        })
                        .on_submit(Message::EnterKeyPressed),
                    text_input("Replace with", &state.replace.0)
                        .id("replace")
                        .on_input_maybe(if !state.confirm {
                            Some(Message::ReplaceChanged)
                        } else {
                            Option::None
                        })
                        .style(|theme, status: text_input::Status| {
                            text_input::default(theme, status).set_border_radius(BORDER_RADIUS)
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
                            .style(|theme, status: text_input::Status| {
                                text_input::default(theme, status).set_border_radius(BORDER_RADIUS)
                            })
                            .on_submit(Message::EnterKeyPressed),
                        button("Browse")
                            .on_press_maybe(if !state.confirm {
                                Some(Message::BrowsePath)
                            } else {
                                Option::None
                            })
                            .style(|theme: &Theme, status: Status| {
                                button::primary(theme, status).set_border_radius(BORDER_RADIUS)
                            })
                            .width(80),
                    ]
                    .spacing(10),
                ]
                .max_width(500)
                .spacing(20),
                //buttons for updating path and running the find and replace operation
                //  will be disabled if state.confirm == true
                column![
                    button("Update path - find")
                        .on_press_maybe(if !state.confirm {
                            Some(Message::UpdatePath(state.find.clone()))
                        } else {
                            Option::None
                        })
                        .width(220)
                        .style(|theme, status| {
                            button::secondary(theme, status).set_border_radius(BORDER_RADIUS)
                        }),
                    button("Update path - replace")
                        .on_press_maybe(if !state.confirm {
                            Some(Message::UpdatePath(state.replace.clone()))
                        } else {
                            Option::None
                        })
                        .width(220)
                        .style(|theme, status| {
                            button::secondary(theme, status).set_border_radius(BORDER_RADIUS)
                        }),
                    button("Find")
                        .on_press_maybe(if !state.confirm {
                            Some(Message::Find)
                        } else {
                            Option::None
                        })
                        .style(|theme: &Theme, status: Status| {
                            button::primary(theme, status).set_border_radius(BORDER_RADIUS)
                        })
                        .width(220),
                ]
                .spacing(20),
            ]
            .spacing(20),
            // Container to display all the actions
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
            .style(|theme| { container::rounded_box(theme).set_border_radius(BORDER_RADIUS) })
            .height(350)
            .width(8000)
            .padding(20),
            // buttons for confirming or cancelling the operation
            //      will be disabled if state.confirm == false
            row![
                button("Replace")
                    .on_press_maybe(if state.confirm {
                        Some(Message::Replace)
                    } else {
                        Option::None
                    })
                    .style(|theme: &Theme, status: Status| {
                        button::primary(theme, status).set_border_radius(BORDER_RADIUS)
                    }),
                button("Cancel")
                    .on_press_maybe(if state.confirm {
                        Some(Message::Cancel)
                    } else {
                        Option::None
                    })
                    .style(|theme: &Theme, status: Status| {
                        button::secondary(theme, status).set_border_radius(BORDER_RADIUS)
                    }),
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
            state.find.0 = find.trim().to_owned();
            Task::none()
        }

        // event handling for the replace text input
        Message::ReplaceChanged(replace) => {
            state.replace.0 = replace.trim().to_owned();
            Task::none()
        }

        // event handling for the directory text input
        Message::ChangePath(dir) => {
            state.path = dir.trim().to_owned();
            state.update_markdown();
            Task::none()
        }

        // event handling for the browse button
        Message::BrowsePath => {
            return Task::perform(AsyncFileDialog::new().pick_folder(), |path| {
                Message::ChangePath(path.unwrap().path().display().to_string())
            });
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

        Message::EnterKeyPressed => {
            if state.confirm {
                return Task::done(Message::Replace);
            }
            Task::done(Message::Find)
        }

        Message::TabKeyPressed => {
            let ids = vec!["replace", "dir", "find"];
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
            width: 1200.0,
            height: 650.0,
        })
        .run()
}
