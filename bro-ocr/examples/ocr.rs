fn main() -> anyhow::Result<()> {
    let file = "./bro-ocr/examples/asserts/image.png";
    let ocr = bro_ocr::OCR {
        path: Some(file.to_string()),
        ..Default::default()
    };
    ocr.run();
    Ok(())
}
