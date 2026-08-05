use std::error::Error;
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
struct ParseFailErr(String);

impl Error for ParseFailErr {}

impl fmt::Display for ParseFailErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

pub fn load_json(json_str: &str) -> Result<Value, Box<dyn Error>> {
    if let json_value = serde_json::from_str(json_str)? {
            return Ok(json_value);
    }

    return Err(Box::new(ParseFailErr("Oops".into())));
}
