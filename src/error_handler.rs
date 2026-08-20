use crate::prelude::{fmt, Error};

#[derive(Debug)]
pub struct ParseFailErr(pub String);

impl Error for ParseFailErr {}

impl fmt::Display for ParseFailErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

