#[derive(Clone, Debug)]
pub enum Error<'a> {
    Infallible(&'a str),
}

impl<'a> std::error::Error for Error<'_> {}

impl<'a> std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Infallible(msg) => write!(f, "{}", msg),
            //_ => write!(f, "Unexplained application error"),
        }
    }
}
