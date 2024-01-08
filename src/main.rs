use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use http::HeaderValue;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, Luma};
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
}
async fn get_qr(Path(_): Path<String>, Query(query): Query<GetQrParameters>) -> impl IntoResponse {
    let size = query.size.unwrap_or(1024).min(4096);
    let data = query.data;
    println!("Generating QR code for '{}' at {}px", data, size);

    let qrcode = QrCode::new(data).unwrap();

    // TODO: Add more options here (colors and quiet zone)
    let image: image::ImageBuffer<Luma<u8>, Vec<u8>> = qrcode
        .render::<Luma<u8>>()
        .min_dimensions(size, size)
        .build();
    let width = image.width();
    let height = image.height();

    // TODO: Support other image formats and SVG
    let mut result_bytes: Vec<u8> = Vec::new();
    let encoder: PngEncoder<&mut Vec<u8>> = PngEncoder::new(&mut result_bytes);
    match encoder.write_image(&image.into_raw(), width, height, ColorType::L8) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to encode image: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), vec![]);
        }
    }

    let mut image_headers = HeaderMap::new();
    image_headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("image/png"),
    );
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
