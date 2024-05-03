use core::fmt;
use iced::futures::StreamExt;
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{
    executor, subscription, theme, Application, Color, Command, Element, Font, Length, Settings,
    Subscription, Theme,
};
use llama_cpp::standard_sampler::StandardSampler;
use llama_cpp::{
    CompletionHandle, LlamaModel, LlamaParams, LlamaSession, SessionParams, TokensToStrings,
};

fn main() -> iced::Result {
    Tic::run(Settings::with_flags(Flags {
        model_path: "./Meta-Llama-3-8B-Instruct.Q4_K_M.gguf".into(),
    }))
}

struct Tic {
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
            text_input("Ask me something", &self.input_buffer)
                .on_submit(Event::Submit)
                .on_input(Event::Input)
        } else {
            text_input("Ask me something", &self.input_buffer).on_submit(Event::Submit)
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
#[derive(Debug)]
struct CompletionRequest {
    id: usize,
    input: String,
    session: LoadedSession,
    state: CompletionRequestState,
}

#[derive(Debug)]
enum CompletionRequestState {
    Processing,
    Done,
    Error,
}

impl CompletionRequest {
    fn new(id: usize, input: &str, session: LoadedSession) -> Self {
        CompletionRequest {
            id,
            input: input.into(),
            session,
            state: CompletionRequestState::Processing,
        }
    }

    fn subscription(&self) -> Subscription<Event> {
        let input = self.input.clone();
        let session = self.session.clone();
        let id = self.id;
        match self.state {
            CompletionRequestState::Processing => {
                subscription::unfold(id, CompletionState::Ready((input, session)), move |state| {
                    run_completion(id, state)
                })
                .map(Event::CompletionResponded)
            }
            CompletionRequestState::Done | CompletionRequestState::Error => Subscription::none(),
        }
    }
}

async fn run_completion(
    id: usize,
    state: CompletionState,
) -> ((usize, CompletionResponse), CompletionState) {
    match state {
        CompletionState::Ready((input, mut session)) => {
            if let Err(e) = session.session.advance_context_async(input).await {
                (
                    (id, CompletionResponse::Error(e.to_string())),
                    CompletionState::Done,
                )
            } else {
                let handle = session
                    .session
                    .start_completing_with(StandardSampler::default(), 1024);
                match handle {
                    Ok(handle) => {
                        let mut stream = handle.into_strings();
                        let first = StreamExt::next(&mut stream).await;
                        match first {
                            Some(first) => (
                                (id, CompletionResponse::Text(first)),
                                CompletionState::Processing(stream),
                            ),
                            None => ((id, CompletionResponse::Done), CompletionState::Done),
                        }
                    }
                    Err(e) => (
                        (id, CompletionResponse::Error(e.to_string())),
                        CompletionState::Done,
                    ),
                }
            }
        }
        CompletionState::Processing(mut stream) => match StreamExt::next(&mut stream).await {
            Some(text) => (
                (id, CompletionResponse::Text(text)),
                CompletionState::Processing(stream),
            ),
            None => ((id, CompletionResponse::Done), CompletionState::Done),
        },
        CompletionState::Done => iced::futures::future::pending().await,
    }
}

enum CompletionState {
    Ready((String, LoadedSession)),
    Processing(TokensToStrings<CompletionHandle>),
    Done,
}

#[derive(Debug, Clone)]
enum CompletionResponse {
    Text(String),
    Error(String),
    Done,
}

#[derive(Debug, Clone)]
enum Event {
    Input(String),
    Submit,
    ModelLoaded(LoadedSession),
    ModelLoadError(String),
    CompletionResponded((usize, CompletionResponse)),
}

#[derive(Clone)]
struct LoadedSession {
    label: String,
    session: LlamaSession,
}

impl fmt::Debug for LoadedSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedSession")
            .field("model", &self.label)
            .finish()
    }
}

struct Flags {
    model_path: String,
}

#[derive(Debug)]
struct MessageContainer {
    text: String,
    sender: Sender,
    completion_request: Option<CompletionRequest>,
}

#[derive(Debug)]
enum Sender {
    User,
    Assistant,
}

impl MessageContainer {
    fn new(message: &str, sender: Sender, completion_request: Option<CompletionRequest>) -> Self {
        MessageContainer {
            text: message.to_string(),
            sender,
            completion_request,
        }
    }

    fn view(&self, default_text_color: Color) -> Element<Event> {
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

    fn subscription(&self) -> Subscription<Event> {
        match &self.completion_request {
            Some(request) => request.subscription(),
            None => Subscription::none(),
        }
    }
}
