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

#[derive(Debug)]
pub enum Flash {
    ERROR,
    INFO,
    WARNING,
    SUCCESS,
}

impl std::fmt::Display for Flash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ERROR => "error",
                Self::INFO => "info",
                Self::WARNING => "warning",
                Self::SUCCESS => "success",
            }
        )
    }
}
