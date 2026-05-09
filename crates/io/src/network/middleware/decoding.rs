use async_compression::tokio::bufread::{BrotliDecoder, GzipDecoder, ZlibDecoder, ZstdDecoder};
use network::{CONTENT_ENCODING, HeaderMap};
use strum::EnumString;
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Encoding {
    Br,
    Deflate,
    Gzip,
    Zstd,
}

pub struct DecodingMiddleware;

impl DecodingMiddleware {
    pub async fn decode(headers: &HeaderMap, data: Vec<u8>) -> Result<Vec<u8>, String> {
        let encoders: Vec<Encoding> = headers
            .get(CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .split(',')
            .filter_map(|encoding| encoding.trim().parse::<Encoding>().ok())
            .collect();

        match encoders.first() {
            Some(encoder) => match encoder {
                Encoding::Br => Self::decode_br(data).await,
                Encoding::Deflate => Self::decode_deflate(data).await,
                Encoding::Gzip => Self::decode_gzip(data).await,
                Encoding::Zstd => Self::decode_zstd(data).await,
            },
            None => Ok(data),
        }
    }

    async fn decode_br(data: Vec<u8>) -> Result<Vec<u8>, String> {
        let reader = BufReader::new(data.as_slice());
        let mut decoder = BrotliDecoder::new(reader);

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .await
            .map_err(|e| e.to_string())?;

        Ok(decompressed)
    }

    async fn decode_deflate(data: Vec<u8>) -> Result<Vec<u8>, String> {
        let reader = BufReader::new(data.as_slice());
        let mut decoder = ZlibDecoder::new(reader);

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .await
            .map_err(|e| e.to_string())?;

        Ok(decompressed)
    }

    async fn decode_gzip(data: Vec<u8>) -> Result<Vec<u8>, String> {
        let reader = BufReader::new(data.as_slice());
        let mut decoder = GzipDecoder::new(reader);

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .await
            .map_err(|e| e.to_string())?;

        Ok(decompressed)
    }

    async fn decode_zstd(data: Vec<u8>) -> Result<Vec<u8>, String> {
        let reader = BufReader::new(data.as_slice());
        let mut decoder = ZstdDecoder::new(reader);

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .await
            .map_err(|e| e.to_string())?;

        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_br() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let input = b"\x0b\x05\x80Hello World\x03";
        let output = runtime
            .block_on(DecodingMiddleware::decode_br(input.to_vec()))
            .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn test_decode_deflate() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let input = [
            0x78, 0x9C, 0xF3, 0x48, 0xCD, 0xC9, 0xC9, 0x57, 0x08, 0xCF, 0x2F, 0xCA, 0x49, 0x01, 0x00, 0x18, 0x0B, 0x04,
            0x1D,
        ];
        let output = runtime
            .block_on(DecodingMiddleware::decode_deflate(input.to_vec()))
            .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn test_decode_gzip() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let input = [
            0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x03, 0xf3, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0x08, 0xcf,
            0x2f, 0xca, 0x49, 0x01, 0x00, 0x56, 0xb1, 0x17, 0x4a, 0x0b, 0x00, 0x00, 0x00,
        ];
        let output = runtime
            .block_on(DecodingMiddleware::decode_gzip(input.to_vec()))
            .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn test_decode_zstd() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let input = [
            0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x58, 0x59, 0x00, 0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72,
            0x6c, 0x64,
        ];
        let output = runtime
            .block_on(DecodingMiddleware::decode_zstd(input.to_vec()))
            .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Hello World".to_string());
    }
}
