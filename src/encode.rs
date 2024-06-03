pub trait Encoder {
    fn encode(&self, content: String) -> String {
        content
    }
}

pub struct Gzip {}

impl Encoder for Gzip {}

pub fn new_encoder(encoder_type: &str) -> Result<Box<dyn Encoder>, &str> {
    match encoder_type {
        "gzip" => Ok(Box::new(Gzip {})),
        _ => Err("encoder not supported"),
    }
}
