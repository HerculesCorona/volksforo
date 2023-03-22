#[derive(askama::Template)]
#[template(path = "util/flash.html")]
pub struct FlashTemplate<'a>(&'a FlashMessage);

#[derive(Debug)]
pub struct FlashMessage {
    pub class: Flash,
    pub message: String,
}

impl std::fmt::Display for FlashMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", FlashTemplate(self))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Flash {
    Error,
    Info,
    Warning,
    Success,
}

impl std::fmt::Display for Flash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Error => "error",
                Self::Info => "info",
                Self::Warning => "warning",
                Self::Success => "success",
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct FlashJar {
    pub messages: Vec<FlashMessage>,
}

impl FlashJar {
    pub fn flash(&mut self, class: Flash, message: &str) {
        self.messages.push(FlashMessage {
            class,
            message: message.to_owned(),
        })
    }
}
