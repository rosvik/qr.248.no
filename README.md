# qr.248.no

A simple Rust-based tool and API that allows you to QR codes.

A QR code to this repository can be found at https://qr.248.no/qr.png?data=https%3A%2F%2Fgithub.com%2Frosvik%2Fqr.248.no&size=500

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
- `size`: The size of the image in pixels. The generator doesn't usually give a image of the exact size requested, but rounds to a larger size where the "QR pixels" are an even amount of "image pixels". _Default: 1000_
- `format`: The format of the generated image. Supports `png`, `jpg`/`jpeg` and `bmp`. The format can either be given as a url param, or as file extension to the route name. (`/image.jpg` or `/image?format=jpg`) _Defaults to `png`_

Example:
```bash
curl "http://127.0.0.1:2339/image-name.png?data=https://example.com/&size=300"
```

<div align="right"><img src="https://github-production-user-asset-6210df.s3.amazonaws.com/1774972/269361517-d0d8e30e-4a25-4ba2-b926-2a42da1156f8.svg" width="32" alt="248"></div>
