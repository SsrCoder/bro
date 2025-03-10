use std::fs;

use clap::Parser;
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};
use reqwest::{blocking::Client, header::HeaderMap};
use serde::Deserialize;
use serde_json::json;

type Error = Box<dyn std::error::Error>;
type Result<T> = core::result::Result<T, Error>;

#[derive(Parser, Debug, Default)]
pub struct Ocr {
    #[arg(short, long, help = "get image from clipboard")]
    pub clipboard: bool,
    #[arg(short, long, help = "write result to clipboard")]
    pub write_clipboard: bool,
    #[arg(short, long)]
    pub path: Option<String>,
    // #[arg(short, long)]
    // pub url: Option<String>,
    #[arg(short, long)]
    pub new_line: Option<bool>,
}

#[derive(Deserialize)]
struct Response<T> {
    // code: i32,
    // msg: String,
    data: T,
}

#[derive(Deserialize)]
struct OCRResultData {
    // hash: Option<String>,
    #[serde(rename = "isEnded")]
    is_ended: bool,
    #[serde(rename = "ydResp")]
    yd_resp: Option<YDResp>,
}

#[derive(Deserialize)]
struct YDResp {
    words_result: Vec<WordsResult>,
    // words_result_num: i32,
    // log_id: i64,
}

#[derive(Deserialize)]
struct WordsResult {
    // location: Location,
    // vertexes_location: Vec<VertexesLocation>,
    words: String,
}

// #[derive(Deserialize)]
// struct Location {
//     top: i32,
//     left: i32,
//     width: i32,
//     height: i32,
// }

// #[derive(Deserialize)]
// struct VertexesLocation {
//     x: i32,
//     y: i32,
// }

struct Image {
    base64: String,
    size: usize,
}

impl Image {
    pub fn new_from_clipboard() -> Result<Image> {
        let img = ClipboardContext::new()
            .unwrap()
            .get_image()
            .or::<&str>(Err("can not get image from clipboard"))?;

        img.save_to_path("image.png")
            .or::<&str>(Err("can not save image from clipboard"))?;
        let img = image::open("image.png")?.to_rgb8();
        img.save("image.jpg")?;

        let image = Image {
            size: fs::metadata("image.jpg")?.len() as usize,
            base64: image_base64::to_base64("image.jpg"),
        };

        fs::remove_file("image.jpg")?;
        Ok(image)
    }

    pub fn new_from_path(path: &str) -> Result<Image> {
        let mut is_temp_file = false;
        let mut path = path;
        if mime_guess::from_path(path).first_or_octet_stream() == mime::IMAGE_PNG {
            let img = image::open(path)?.to_rgb8();
            img.save("image.jpg")?;
            path = "image.jpg";
            is_temp_file = true;
        }

        let size = fs::metadata(path)?.len();
        let image = image_base64::to_base64(path);
        if is_temp_file {
            fs::remove_file(path)?;
        }

        Ok(Image {
            size: size as usize,
            base64: image,
        })
    }
}

impl Ocr {
    pub fn run(&self) {
        self.handle().unwrap()
    }

    pub fn handle(&self) -> Result<()> {
        let image = self.get_image()?;

        let client = self.new_client()?;
        let (token, engine) = self.get_upload_token_and_engine(&client)?;
        let job_id = self.upload_image(&client, &token, &engine, &image)?;
        let yd_resp = self.get_result(&client, &job_id, &engine)?;

        let res = yd_resp.words_result.iter().fold(String::new(), |acc, x| {
            if self.new_line.unwrap_or_default() {
                format!("{acc}{}\n", x.words)
            } else {
                format!("{acc}{}", x.words)
            }
        });
        println!("{res}");

        if self.write_clipboard {
            ClipboardContext::new().unwrap().set_text(res).unwrap();
        }

        Ok(())
    }

    fn get_result(
        &self,
        client: &reqwest::blocking::Client,
        job_id: &str,
        engine: &str,
    ) -> Result<YDResp> {
        for _ in 0..30 {
            let res = client
                .get(format!(
                    "https://web.baimiaoapp.com/api/ocr/image/{engine}/status?jobStatusId={job_id}"
                ))
                .send()?
                .json::<Response<OCRResultData>>()?;
            if res.data.is_ended && res.data.yd_resp.is_some() {
                return Ok(res.data.yd_resp.unwrap());
            }
        }
        Err("timeout".into())
    }

    fn get_image(&self) -> Result<Image> {
        if let Some(path) = &self.path {
            Ok(Image::new_from_path(path)?)
        } else if self.clipboard {
            Ok(Image::new_from_clipboard()?)
        } else {
            Err("can not parse image".into())
        }
    }

    fn upload_image(
        &self,
        client: &reqwest::blocking::Client,
        token: &str,
        engine: &str,
        image: &Image,
    ) -> Result<String> {
        let body = json!({
            "token": token,
            "hash": "",
            "name": "image.png",
            "size": image.size,
            "dataUrl": image.base64,
            "result": {},
            "status": "processing",
            "isSuccess": false
        })
        .to_string();

        #[derive(Deserialize)]
        struct ResponseData {
            // hash: Option<String>,
            #[serde(rename = "jobStatusId")]
            job_status_id: String,
        }

        let res = client
            .post(format!("https://web.baimiaoapp.com/api/ocr/image/{engine}"))
            .body(body)
            .send()?
            .json::<Response<ResponseData>>()?;

        Ok(res.data.job_status_id)
    }

    fn get_upload_token_and_engine(
        &self,
        client: &reqwest::blocking::Client,
    ) -> Result<(String, String)> {
        #[derive(Deserialize)]
        struct ResponseData {
            token: String,
            engine: String,
        }

        let body = json! ({"mode": "single"});
        let resp = client
            .post("https://web.baimiaoapp.com/api/perm/single")
            .body(body.to_string())
            .send()?
            .json::<Response<ResponseData>>()?;
        Ok((resp.data.token, resp.data.engine))
    }

    fn new_client(&self) -> Result<reqwest::blocking::Client> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let token = self.get_auth_token(&uuid)?;

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0".parse()?);
        headers.insert("X-Auth-Token", token.parse()?);
        headers.insert("X-Auth-Uuid", uuid.parse()?);
        Ok(Client::builder().default_headers(headers).build()?)
    }

    fn get_auth_token(&self, uuid: &str) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0".parse()?);
        headers.insert("X-Auth-Uuid", uuid.parse()?);

        #[derive(Deserialize)]
        struct Token {
            token: String,
        }

        let res = Client::builder()
            .default_headers(headers)
            .build()?
            .post("https://web.baimiaoapp.com/api/user/login/anonymous")
            .send()?
            .json::<Response<Token>>()?;
        Ok(res.data.token)
    }
}
