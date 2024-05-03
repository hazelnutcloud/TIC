use iced::{theme, widget::text, Color, Element, Font, Subscription};

use crate::{app::Event, completions::CompletionRequest};

#[derive(Debug)]
pub struct MessageContainer {
    pub text: String,
    pub sender: Sender,
    pub completion_request: Option<CompletionRequest>,
}

#[derive(Debug)]
pub enum Sender {
    User,
    Assistant,
}

impl MessageContainer {
    pub fn new(
        message: &str,
        sender: Sender,
        completion_request: Option<CompletionRequest>,
    ) -> Self {
        MessageContainer {
            text: message.to_string(),
            sender,
            completion_request,
        }
    }

    pub fn view(&self, default_text_color: Color) -> Element<Event> {
        let color = match self.sender {
            Sender::User => default_text_color,
            Sender::Assistant => Color::from_rgb(
                default_text_color.r - 0.2,
                default_text_color.g - 0.2,
                default_text_color.b,
            ),
        };
        text(self.text.clone())
            .font(Font::MONOSPACE)
            .style(theme::Text::Color(color))
            .size(12)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Event> {
        match &self.completion_request {
            Some(request) => request.subscription(),
            None => Subscription::none(),
        }
    }
}
