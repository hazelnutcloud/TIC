use iced::Application;
use iced::Settings;
use tic::{Flags, Tic};

fn main() -> iced::Result {
    Tic::run(Settings::with_flags(Flags {
        model_path: "./Meta-Llama-3-8B-Instruct.Q4_K_M.gguf".into(),
    }))
}
