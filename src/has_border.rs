use iced::{
    border,
    widget::{button, container, text_input},
};

pub trait HasBorder {
    fn set_border_radius(&mut self, radius: f32) -> Self;
}

impl HasBorder for button::Style {
    fn set_border_radius(&mut self, radius: f32) -> button::Style {
        let mut clone = self.clone();
        clone.border.radius = border::radius(radius);
        clone
    }
}

impl HasBorder for text_input::Style {
    fn set_border_radius(&mut self, radius: f32) -> text_input::Style {
        let mut clone = self.clone();
        clone.border.radius = border::radius(radius);
        clone
    }
}

impl HasBorder for container::Style {
    fn set_border_radius(&mut self, radius: f32) -> container::Style {
        let mut clone = self.clone();
        clone.border.radius = border::radius(radius);
        clone
    }
}
