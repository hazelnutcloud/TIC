use iced::widget::{column, container, scrollable, text_input};
use iced::{executor, Application, Command, Element, Length, Subscription, Theme};
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};

use crate::{
    completions::{CompletionRequest, CompletionRequestState, CompletionResponse, LoadedSession},
    components::message::{MessageContainer, Sender},
};

pub struct Flags {
    pub model_path: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    Input(String),
    Submit,
    ModelLoaded(LoadedSession),
    ModelLoadError(String),
    CompletionResponded((usize, CompletionResponse)),
}

pub struct Tic {
    input_buffer: String,
    message_containers: Vec<MessageContainer>,
    session: Option<LoadedSession>,
    request_id: usize,
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
                    Sender::Assistant,
                    None,
                )],
                input_buffer: String::new(),
                session: None,
                request_id: 0,
            },
            Command::perform(
                LlamaModel::load_from_file_async(path.clone(), LlamaParams::default()),
                move |load_result| match load_result {
                    Ok(model) => {
                        if let Ok(session) = model.create_session(SessionParams::default()) {
                            Event::ModelLoaded(LoadedSession {
                                label: path.clone(),
                                session,
                            })
                        } else {
                            Event::ModelLoadError("Error creating session".into())
                        }
                    }
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
                if self.session.is_none() || self.input_buffer.trim().is_empty() {
                    return Command::none();
                }

                let input = self.input_buffer.clone();
                let completion_request =
                    CompletionRequest::new(self.request_id, &input, self.session.clone().unwrap());

                self.message_containers.push(MessageContainer::new(
                    &self.input_buffer,
                    Sender::User,
                    None,
                ));
                self.message_containers.push(MessageContainer::new(
                    "...",
                    Sender::Assistant,
                    Some(completion_request),
                ));
                self.input_buffer.clear();
                self.request_id += 1;
            }
            Event::ModelLoaded(session) => {
                self.session = Some(session);
            }
            Event::ModelLoadError(e) => eprintln!("Error loading model: {}", e),
            Event::CompletionResponded((id, response)) => match response {
                CompletionResponse::Text(text) => {
                    if let Some(container) = self.message_containers.iter_mut().find(|container| {
                        if let Some(request) = &container.completion_request {
                            return request.id == id;
                        }
                        false
                    }) {
                        container.text.push_str(&text);
                    } else {
                        eprintln!("No completion request found for id {}", id);
                    }
                }
                CompletionResponse::Error(e) => {
                    eprintln!("Error completing: {}", e);
                    if let Some(container) = self.message_containers.iter_mut().find(|container| {
                        if let Some(request) = &container.completion_request {
                            return request.id == id;
                        }
                        false
                    }) {
                        container.completion_request.as_mut().unwrap().state =
                            CompletionRequestState::Error;
                    }
                }
                CompletionResponse::Done => {
                    if let Some(container) = self.message_containers.iter_mut().find(|container| {
                        if let Some(request) = &container.completion_request {
                            return request.id == id;
                        }
                        false
                    }) {
                        container.completion_request.as_mut().unwrap().state =
                            CompletionRequestState::Done;
                    }
                }
            },
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if self.session.is_none() {
            return Subscription::none();
        }
        Subscription::batch(
            self.message_containers
                .iter()
                .map(MessageContainer::subscription),
        )
    }

    fn view(&self) -> Element<Self::Message> {
        let text_color = self.theme().palette().text;
        let messages = column![]
            .extend(
                self.message_containers
                    .iter()
                    .map(|container| container.view(text_color)),
            )
            .spacing(5);
        let messages = scrollable(messages).height(Length::Fill);
        let input = if self.session.is_some() {
            text_input("How can I help you?", &self.input_buffer)
                .on_submit(Event::Submit)
                .on_input(Event::Input)
        } else {
            text_input("How can I help you?", &self.input_buffer).on_submit(Event::Submit)
        };
        container(column![messages, input].spacing(10.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Nord
    }
}
