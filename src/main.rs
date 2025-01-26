use axum::extract::{Path, Query};
use axum::http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::Router;
use image::codecs::{bmp::BmpEncoder, jpeg::JpegEncoder, png::PngEncoder};
use image::{ColorType, ImageEncoder, ImageFormat, Luma};
use qrcode::QrCode;
use serde::{Deserialize, Deserializer};

const ADDR: &str = "0.0.0.0:2339";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", axum::routing::get(index))
        .route("/{filename}", axum::routing::get(get_qr));

    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    println!("Listening on http://{}", ADDR);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

#[derive(Deserialize)]
struct GetQrParameters {
    data: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    size: Option<u32>,
    format: Option<String>,
}
async fn get_qr(
    Path(filename): Path<String>,
    Query(query): Query<GetQrParameters>,
) -> impl IntoResponse {
    let size = query.size.unwrap_or(1024).min(4096);
    let data = query.data;
    let format = query.format;
    println!("Generating QR code for '{}' at {}px", data, size);

    let is_svg = match format.clone() {
        Some(param) => param == "svg",
        None => filename.ends_with(".svg"),
    };
    if is_svg {
        let (status_code, headers, result_bytes) = get_svg(&data, size);
        return (status_code, headers, result_bytes);
    }

    let qrcode = QrCode::new(data).unwrap();

    // TODO: Add more options here (colors and quiet zone)
    let image: image::ImageBuffer<Luma<u8>, Vec<u8>> = qrcode
        .render::<Luma<u8>>()
        .min_dimensions(size, size)
        .build();

    let image_format = match get_format_from_filename(match format {
        Some(param) => format!(".{}", param),
        None => "".into(),
    }) {
        Some(format) => format,
        None => get_format_from_filename(filename).unwrap_or(ImageFormat::Png),
    };

    let (result_bytes, header_value) = encode_image(image, image_format);

    let mut image_headers = HeaderMap::new();
    image_headers.insert(CONTENT_TYPE, header_value);

    (StatusCode::OK, image_headers, result_bytes)
}

fn get_svg(data: &String, size: u32) -> (StatusCode, HeaderMap, Vec<u8>) {
    println!("Data: {}", data);
    let code = match QrCode::new(data) {
        Ok(code) => code,
        Err(e) => {
            println!("Error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                HeaderMap::new(),
                "Invalid data".as_bytes().to_vec(),
            );
        }
    };
    // TODO: Add color options
    let image = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(size, size)
        .build();

    println!("{}", image);

    let mut image_headers = HeaderMap::new();
    image_headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));

    (StatusCode::OK, image_headers, image.as_bytes().to_vec())
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => std::str::FromStr::from_str(s)
            .map_err(serde::de::Error::custom)
            .map(Some),
    }
}

fn get_format_from_filename(filename: String) -> Option<ImageFormat> {
    if filename.to_lowercase().ends_with(".png") {
        Some(ImageFormat::Png)
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        Some(ImageFormat::Jpeg)
    } else if filename.ends_with(".gif") {
        Some(ImageFormat::Gif)
    } else if filename.ends_with(".bmp") {
        Some(ImageFormat::Bmp)
    } else if filename.ends_with(".ico") {
        Some(ImageFormat::Ico)
    } else if filename.ends_with(".tiff") || filename.ends_with(".tif") {
        Some(ImageFormat::Tiff)
    } else if filename.ends_with(".webp") {
        Some(ImageFormat::WebP)
    } else {
        None
    }
}

fn encode_image(
    image: image::ImageBuffer<Luma<u8>, Vec<u8>>,
    format: ImageFormat,
) -> (Vec<u8>, HeaderValue) {
    let w = image.width();
    let h = image.height();
    let color_type = ColorType::L8;

    let mut result_bytes: Vec<u8> = Vec::new();

    let header_value: HeaderValue = match format {
        ImageFormat::Jpeg => {
            JpegEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/jpeg")
        }
        ImageFormat::Bmp => {
            BmpEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/bmp")
        }
        // TODO: ImageFormat::Tiff
        // TODO: ImageFormat::Gif
        _ => {
            PngEncoder::new(&mut result_bytes)
                .write_image(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/png")
        }
    };
    (result_bytes, header_value)
}
