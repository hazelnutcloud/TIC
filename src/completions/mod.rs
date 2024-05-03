use core::fmt;

use iced::{futures::StreamExt, subscription, Subscription};
use llama_cpp::{
    standard_sampler::StandardSampler, CompletionHandle, LlamaSession, TokensToStrings,
};

use crate::app::Event;

pub mod chat;

#[derive(Debug)]
pub struct CompletionRequest {
    pub id: usize,
    pub input: String,
    pub session: LoadedSession,
    pub state: CompletionRequestState,
}

#[derive(Debug)]
pub enum CompletionRequestState {
    Processing,
    Done,
    Error,
}

impl CompletionRequest {
    pub fn new(id: usize, input: &str, session: LoadedSession) -> Self {
        CompletionRequest {
            id,
            input: input.into(),
            session,
            state: CompletionRequestState::Processing,
        }
    }

    pub fn subscription(&self) -> Subscription<Event> {
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

pub async fn run_completion(
    id: usize,
    state: CompletionState,
) -> ((usize, CompletionResponse), CompletionState) {
    match state {
        CompletionState::Ready((input, mut session)) => {
            if let Err(e) = session.session.set_context_async(input).await {
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
                                CompletionState::Processing((stream, None)),
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
        CompletionState::Processing((mut stream, eos_check)) => {
            println!("eos_check: {:?}", eos_check);
            if let Some(maybe_eos) = &eos_check {
                if maybe_eos.starts_with("<|eot_id|>") {
                    return ((id, CompletionResponse::Done), CompletionState::Done);
                }
            }
            let mut eos_check = eos_check.clone();
            loop {
                match StreamExt::next(&mut stream).await {
                    Some(text) => {
                        println!("text: {:?}", text);
                        eos_check = match eos_check {
                            Some(partial_eos) => {
                                let partial_eos = partial_eos + &text;
                                ("<|eot_id|>".starts_with(&partial_eos)
                                    || partial_eos.starts_with("<|eot_id|>"))
                                .then_some(partial_eos)
                            }
                            None => text
                                .contains("<")
                                .then(|| text.get(text.find("<").unwrap()..).unwrap().to_string()),
                        };
                        if eos_check.is_some() {
                            continue;
                        }
                        return (
                            (id, CompletionResponse::Text(text)),
                            CompletionState::Processing((stream, eos_check)),
                        );
                    }
                    None => return ((id, CompletionResponse::Done), CompletionState::Done),
                }
            }
        }
        CompletionState::Done => iced::futures::future::pending().await,
    }
}

pub enum CompletionState {
    Ready((String, LoadedSession)),
    Processing((TokensToStrings<CompletionHandle>, Option<String>)),
    Done,
}

#[derive(Debug, Clone)]
pub enum CompletionResponse {
    Text(String),
    Error(String),
    Done,
}

#[derive(Clone)]
pub struct LoadedSession {
    pub label: String,
    pub session: LlamaSession,
}

impl fmt::Debug for LoadedSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedSession")
            .field("model", &self.label)
            .finish()
    }
}
