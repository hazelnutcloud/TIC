use minijinja::{context, Environment};

use crate::components::message::{MessageContainer, Sender};

pub struct ChatTemplater {
    env: Environment<'static>,
}

impl ChatTemplater {
    pub fn new() -> Result<ChatTemplater, minijinja::Error> {
        let mut env = Environment::new();
        env.add_template("llama3", include_str!("templates/llama3.jinja"))?;
        Ok(ChatTemplater { env })
    }

    pub fn apply(
        &self,
        template: ChatTemplate,
        conversation: Vec<ChatMessage>,
    ) -> Result<String, minijinja::Error> {
        match template {
            ChatTemplate::Llama3 => {
                let tmpl = self.env.get_template("llama3")?;
                let rendered = tmpl.render(context! {
                  messages => conversation.iter().map(|message| {
                    context! {
                      content => message.text,
                      role => match message.sender {
                          Sender::User => "user",
                          Sender::Assistant => "assistant",
                          Sender::System => "system",
                      }
                    }
                  }).collect::<Vec<_>>(),
                  bos_token => "<|begin_of_text|>",
                  add_generation_prompt => true
                })?;
                Ok(rendered)
            }
        }
    }
}

pub enum ChatTemplate {
    Llama3,
}

pub struct ChatMessage {
    pub text: String,
    pub sender: Sender,
}

impl From<&MessageContainer> for ChatMessage {
    fn from(container: &MessageContainer) -> Self {
        ChatMessage {
            text: container.text.clone(),
            sender: container.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_templater() {
        let templater = ChatTemplater::new().unwrap();
        let conversation = vec![
            ChatMessage {
                text: "Hello".into(),
                sender: Sender::User,
            },
            ChatMessage {
                text: "Hi".into(),
                sender: Sender::Assistant,
            },
        ];
        let rendered = templater.apply(ChatTemplate::Llama3, conversation).unwrap();

        assert_eq!(
            rendered,
            r#"<|begin_of_text|><|start_header_id|>user<|end_header_id|>

Hello<|eot_id|><|start_header_id|>assistant<|end_header_id|>

Hi<|eot_id|><|start_header_id|>assistant<|end_header_id|>

"#
        );
    }
}
