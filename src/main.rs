use core::fmt;

use iced::widget::{column, text, text_input, vertical_space};
use iced::{executor, Application, Command, Element, Font, Length, Settings, Theme};
use llama_cpp::{LlamaModel, LlamaParams};

struct Tic {
    input_buffer: String,
    message_containers: Vec<MessageContainer>,
    llama_model: Option<LlamaModel>,
}

#[derive(Debug, Clone)]
enum Event {
    Input(String),
    Submit,
    ModelLoaded(LabelledLlamaModel),
    ModelLoadError(String),
}

#[derive(Clone)]
struct LabelledLlamaModel {
    label: String,
    model: LlamaModel,
}

impl From<LlamaModel> for LabelledLlamaModel {
    fn from(model: LlamaModel) -> Self {
        LabelledLlamaModel {
            label: "Llama Model".into(),
            model,
        }
    }
}

impl fmt::Debug for LabelledLlamaModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LabelledLlamaModel")
            .field("model", &self.label)
            .finish()
    }
}

struct Flags {
    model_path: String,
}

impl Application for Tic {
    type Executor = executor::Default;

    type Theme = Theme;

    type Flags = Flags;

    type Message = Event;

    fn new(flags: Self::Flags) -> (Tic, iced::Command<Event>) {
        let path = flags.model_path.clone();
        (
            Tic {
                message_containers: vec![MessageContainer::new(
                    "Hello, world!",
                    MessageType::AssistantMessage,
                )],
                input_buffer: String::new(),
                llama_model: None,
            },
            Command::perform(
                LlamaModel::load_from_file_async(path, LlamaParams::default()),
                |load_result| match load_result {
                    Ok(model) => Event::ModelLoaded(LabelledLlamaModel::from(model)),
                    Err(err) => Event::ModelLoadError(err.to_string()),
                },
            ),
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
            Event::ModelLoaded(model) => {
                self.llama_model = Some(model.model);
            }
            Event::ModelLoadError(e) => eprintln!("Error loading model: {}", e),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let messages = column(self.message_containers.iter().map(MessageContainer::view));
        column![
            vertical_space(),
            messages,
            text_input("Ask me something", &self.input_buffer)
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

#[derive(Debug)]
struct MessageContainer {
    value: String,
    message_type: MessageType,
}

#[derive(Debug)]
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
    Tic::run(Settings::with_flags(Flags {
        model_path: "./Meta-Llama-3-8B-Instruct.Q4_K_M.gguf".into(),
    }))
}
