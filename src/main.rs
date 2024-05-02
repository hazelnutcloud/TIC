use iced::widget::text;
use iced::{executor, Application, Command, Font, Settings, Theme};

#[derive(Debug, Default)]
struct Tic {}

#[derive(Debug, Clone)]
enum Message {}

impl Application for Tic {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Tic::default(), Command::none())
    }

    fn title(&self) -> String {
        "TIC - Text Inference Companion".into()
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        text("Hello, world!").font(Font::MONOSPACE).into()
    }
}

fn main() -> iced::Result {
    Tic::run(Settings::default())
}
