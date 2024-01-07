# qr.248.no

A simple Rust-based tool and API that allows you to QR codes. The API uses the Axum framework for handling HTTP requests and responses and the Serde library for deserializing query parameters.

## How to Use

### Setup

```bash
cargo run
```

The API will start running on `http://127.0.0.1:2339`.

### Usage

`GET /<FILENAME>?data=<DATA>&size=<PIXELS>`

Parameters:
- `data`: The data to be encoded in the QR code.
- `size`: The size of the QR code in pixels. _Default: 1000_

Example:
```bash
curl "http://127.0.0.1:2339/image-name.jpg?data=https://example.com/&size=300"
```

<div align="right"><img src="https://github-production-user-asset-6210df.s3.amazonaws.com/1774972/269361517-d0d8e30e-4a25-4ba2-b926-2a42da1156f8.svg" width="32" alt="248"></div>
