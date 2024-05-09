use iced::Application;
use iced::Settings;
use tic::{ChatTemplate, Flags, Tic};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    Tic::run(Settings::with_flags(Flags {
        model_path: "./kappa-3-phi-3-4k-instruct-abliterated-f16.gguf".into(),
        chat_template: ChatTemplate::Phi3,
    }))
}
