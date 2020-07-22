use image::{ColorType, DynamicImage, ImageOutputFormat};
use structopt::StructOpt;

#[derive(Debug)]
struct RgbColor<T> {
    r: T,
    g: T,
    b: T,
}

impl RgbColor<u8> {
    fn to_string(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("rgb({},{},{})", self.r, self.g, self.b));
        buffer
    }
}

#[derive(StructOpt)]
struct Cli {
    image_url: String,
}

fn scale_down_by_width(width: &f32, height: &f32, new_width: &f32) -> f32 {
    let aspect_ratio = width / height;
    new_width / aspect_ratio
}

fn take_format<'a>(mime: &'a str, image: &'a DynamicImage) -> ImageOutputFormat {
    let result = match mime {
        "image/jpeg" => ImageOutputFormat::Jpeg(std::mem::size_of_val(image) as u8),
        "image/png" => ImageOutputFormat::Png,
        "image/gif" => ImageOutputFormat::Gif,
        _ => ImageOutputFormat::Unsupported(String::from(mime)),
    };

    result
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::from_args();
    let response = reqwest::get(&args.image_url).await?.bytes().await?;

    let image = image::load_from_memory(&response).unwrap();
    let metadata = immeta::load_from_buf(&response).unwrap();
    let mime_type = metadata.mime_type();
    let width = metadata.dimensions().width;
    let height = metadata.dimensions().height;

    let has_alpha = match image.color() {
        ColorType::Rgba8 => true,
        ColorType::Bgra8 => true,
        _ => false,
    };

    let colors = dominant_color::get_colors(&response, has_alpha);
    let mut rgb_colors: Vec<RgbColor<u8>> = Vec::new();
    for n in (2..colors.len()).step_by(3) {
        rgb_colors.push(RgbColor {
            r: colors[n - 2],
            g: colors[n - 1],
            b: colors[n],
        })
    }
    let rgb_colors: Vec<String> = rgb_colors
        .into_iter()
        .map(|color| color.to_string())
        .collect();

    let thumbnail_width = 5;
    let thumbnail_height =
        scale_down_by_width(&(width as f32), &(height as f32), &(thumbnail_width as f32));
    let thumbnail = image.thumbnail((thumbnail_width) as u32, (thumbnail_height) as u32);
    let mut buff = vec![];
    let format = take_format(mime_type, &thumbnail);
    thumbnail.write_to(&mut buff, format).unwrap();
    let thumbnail_base64 = base64::encode(&buff);

    println!(
        "color: {:?}
original_dimension: {}/{}
lowest_dimension: {}/{}
base64_thumbnail: data:{};base64,{}",
        rgb_colors, width, height, thumbnail_width, thumbnail_height, mime_type, thumbnail_base64
    );

    Ok(())
}
