use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use http::HeaderValue;
use image::{codecs::*, ColorType, ImageEncoder, ImageFormat, Luma};
use qrcode::QrCode;
use serde::{de, Deserialize, Deserializer};
use std::{fmt, net::SocketAddr, str::FromStr};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/:filename", get(get_qr));

    let addr = SocketAddr::from(([127, 0, 0, 1], 2339));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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
    println!("Generating QR code for '{}' at {}px", data, size);

    let qrcode = QrCode::new(data).unwrap();

    // TODO: Add more options here (colors and quiet zone)
    let image: image::ImageBuffer<Luma<u8>, Vec<u8>> = qrcode
        .render::<Luma<u8>>()
        .min_dimensions(size, size)
        .build();

    let image_format = match get_format_from_filename(match query.format {
        Some(param) => format!(".{}", param),
        None => "".into(),
    }) {
        Some(format) => format,
        None => get_format_from_filename(filename).unwrap_or(ImageFormat::Png),
    };

    let (result_bytes, header_value) = encode_image(image, image_format);

    let mut image_headers = HeaderMap::new();
    image_headers.insert(http::header::CONTENT_TYPE, header_value);

    (StatusCode::OK, image_headers, result_bytes)
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
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
            jpeg::JpegEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/jpeg")
        }
        ImageFormat::Bmp => {
            bmp::BmpEncoder::new(&mut result_bytes)
                .encode(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/bmp")
        }
        // TODO: ImageFormat::Tiff
        // TODO: ImageFormat::Gif
        _ => {
            png::PngEncoder::new(&mut result_bytes)
                .write_image(&image.into_raw(), w, h, color_type)
                .unwrap();
            HeaderValue::from_static("image/png")
        }
    };
    (result_bytes, header_value)
}
