use iced::theme::TextInput;
use iced::widget::{column, text, text_input, vertical_space};
use iced::{executor, Application, Command, Element, Font, Length, Settings, Theme};

#[derive(Debug, Default)]
struct Tic {
    input_buffer: String,
    message_containers: Vec<MessageContainer>,
}

#[derive(Debug, Clone)]
enum Event {
    Input(String),
    Submit,
}

impl Application for Tic {
    type Executor = executor::Default;

    type Theme = Theme;

    type Flags = ();

    type Message = Event;

    fn new(_flags: Self::Flags) -> (Tic, iced::Command<Event>) {
        (
            Tic {
                message_containers: vec![MessageContainer::new(
                    "Hello, world!",
                    MessageType::AssistantMessage,
                )],
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "TIC - Text Inference Companion".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Event> {
        match message {
            Event::Input(input) => {
                self.input_buffer = input;
            }
            Event::Submit => {
                self.message_containers.push(MessageContainer::new(
                    &self.input_buffer,
                    MessageType::UserMessage,
                ));
                self.input_buffer.clear();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let messages = column(self.message_containers.iter().map(MessageContainer::view));
        column![
            vertical_space(),
            messages,
            text_input("Ask me something", &self.input_buffer)
                .style(TextInput::Custom(Box::new(InputStyle::default())))
                .on_input(Event::Input)
                .on_submit(Event::Submit),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Nord
    }
}

#[derive(Default)]
struct InputStyle {
  default_style: iced::theme::TextInput
}

impl text_input::StyleSheet for InputStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        style.active(&self.default_style)
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        style.focused(&self.default_style)
    }

    fn placeholder_color(&self, style: &Self::Style) -> iced::Color {
        style.placeholder_color(&self.default_style)
    }

    fn value_color(&self, style: &Self::Style) -> iced::Color {
        style.value_color(&self.default_style)
    }

    fn disabled_color(&self, style: &Self::Style) -> iced::Color {
        style.disabled_color(&self.default_style)
    }

    fn selection_color(&self, style: &Self::Style) -> iced::Color {
        style.selection_color(&self.default_style)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        style.disabled(&self.default_style)
    }
}

#[derive(Debug, Clone)]
struct MessageContainer {
    value: String,
    message_type: MessageType,
}

#[derive(Debug, Clone)]
enum MessageType {
    UserMessage,
    AssistantMessage,
}

impl MessageContainer {
    fn new(message: &str, message_type: MessageType) -> Self {
        MessageContainer {
            value: message.to_string(),
            message_type,
        }
    }

    fn view(&self) -> Element<Event> {
        text(self.value.clone())
            .font(Font::MONOSPACE)
            .size(12)
            .into()
    }
}

fn main() -> iced::Result {
    Tic::run(Settings::default())
}
