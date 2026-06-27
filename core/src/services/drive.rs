use image::DynamicImage;
use std::io::Cursor;

pub fn process_image(data: &[u8], mime_type: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    if !mime_type.starts_with("image/") {
        return None;
    }

    let img = image::load_from_memory(data).ok()?;

    let webp_data = encode_webp(&img)?;

    let thumbnail = img.thumbnail(400, 400);
    let thumb_data = encode_webp(&thumbnail)?;

    Some((webp_data, thumb_data))
}

fn encode_webp(img: &DynamicImage) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder =
        image::codecs::webp::WebPEncoder::new_lossless(Cursor::new(&mut buf));
    encoder
        .encode(img.as_bytes(), img.width(), img.height(), img.color().into())
        .ok()?;
    Some(buf)
}
