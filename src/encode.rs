use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub trait Encoder {
    fn encode(&self, content: String) -> Vec<u8>;
}

pub struct Gzip {}

impl Encoder for Gzip {
    fn encode(&self, content: String) -> Vec<u8> {
        let mut encoder = GzEncoder::new(vec![], Compression::default());
        let content_buf = content.as_bytes();
        let _ = encoder.write_all(content_buf);

        match encoder.finish() {
            Ok(encoded_data) => {
                return encoded_data;
            }
            Err(err) => {
                eprintln!("{}", err);
                return content.as_bytes().to_vec();
            }
        }
    }
}

pub fn new_encoder(encoder_type: &str) -> Result<Box<dyn Encoder>, &str> {
    match encoder_type {
        "gzip" => Ok(Box::new(Gzip {})),
        _ => Err("encoder not supported"),
    }
}
