use std::{error::Error, fs, io::Read};

use base64::Engine;
use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct OCR {
    #[arg(short, long)]
    pub path: Option<String>,
    #[arg(short, long)]
    pub url: Option<String>,
}

static BAIMIAO_TOKEN: &str = "69AzHvFY7tysTbdXPi1VJWoEx1ziKrhNRtr0OiH8BpPKATalKmXdIH2xqnvxqvSH";
static BAIMIAO_UUID: &str = "55721285-9678-43bb-89b8-6878453760dc";

impl OCR {
    pub fn run(&self) {
        self.handle().unwrap()
    }

    pub fn handle(&self) -> Result<(), Box<dyn Error>> {
        if let Some(path) = &self.path {
            let file = fs::read(path)?;
            let len = file.len();

            let res = base64::prelude::BASE64_STANDARD.encode(file);
            println!("{self:?}: Len: {len}, res: {res}");
        }
        println!("{self:?}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load() {
        let file = "/workspaces/bro/bro-ocr/examples/asserts/image.png";
        let ocr = super::OCR {
            path: Some(file.to_string()),
            ..Default::default()
        };
        ocr.run();
    }
}
