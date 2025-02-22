use iced::{advanced::Widget, widget::TextInput, Renderer, Task, Theme};

use crate::Message;

pub trait Focused<'a, Message, Theme, Render> {
    fn focused<T>(&self) -> Task<T>;
}

impl<'a> Focused<'a, Message, Theme, Renderer> for TextInput<'_, Message, Theme, Renderer> {
    fn focused<T>(&self) -> Task<T> {
        Task::none()
    }
}
