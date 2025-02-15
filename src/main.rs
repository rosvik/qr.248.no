mod deserializers;

use crate::deserializers::{empty_string_as_none, string_as_bool};
use axum::extract::{Path, Query};
use axum::http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::Router;
use image::codecs::{bmp::BmpEncoder, jpeg::JpegEncoder, png::PngEncoder};
use image::{ColorType, ImageEncoder, ImageFormat, Luma};
use qrcode::QrCode;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or("2339".to_string());
    let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .route("/", axum::routing::get(index))
        .route("/{filename}", axum::routing::get(get_qr));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", addr);
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
    #[serde(default, deserialize_with = "string_as_bool")]
    base64: bool,
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

    let result_bytes = encode_image(image, image_format);
    let mut image_headers = HeaderMap::new();

    if query.base64 {
        let b64_data = to_base64(&result_bytes, image_format);
        image_headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        return (StatusCode::OK, image_headers, b64_data.into());
    }

    image_headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(image_format.to_mime_type()),
    );

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

fn encode_image(image: image::ImageBuffer<Luma<u8>, Vec<u8>>, format: ImageFormat) -> Vec<u8> {
    let w = image.width();
    let h = image.height();
    let color_type = ColorType::L8;

    let mut result_bytes: Vec<u8> = Vec::new();

    match format {
        ImageFormat::Jpeg => {
            JpegEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
        }
        ImageFormat::Bmp => {
            BmpEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
        }
        // TODO: ImageFormat::Tiff
        // TODO: ImageFormat::Gif
        _ => {
            PngEncoder::new(&mut result_bytes)
                .write_image(&image.into_raw(), w, h, color_type)
                .unwrap();
        }
    };
    result_bytes
}

use base64::Engine as _;
const B64_ENGINE: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::NO_PAD,
);
pub fn to_base64(buffer: &Vec<u8>, image_format: ImageFormat) -> String {
    let base64_data = B64_ENGINE.encode(buffer);
    format!(
        "data:{};base64,{}",
        image_format.to_mime_type(),
        base64_data
    )
}
