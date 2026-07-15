use std::io;

use async_compression::tokio::bufread::{BrotliDecoder, GzipDecoder, ZlibDecoder, ZstdDecoder};
use bytes::Bytes;
use futures::TryStreamExt;
use http::{HeaderMap, header::CONTENT_ENCODING};
use strum::EnumString;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::trace;

use http_types::body::BodyStream;

use crate::errors::NetworkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Encoding {
    Br,
    Deflate,
    Gzip,
    Zstd,
}

pub(crate) fn get_encoding_order(headers: &HeaderMap) -> Result<Vec<Encoding>, NetworkError> {
    if let Some(encoder_value) = headers.get(CONTENT_ENCODING) {
        Ok(encoder_value
            .to_str()
            .map_err(|e| NetworkError::DecodingError(format!("Header string conversion error, {e}")))?
            .split(',')
            .map(|e| e.trim().parse::<Encoding>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NetworkError::DecodingError(format!("Header parsing error, {e}")))?
            .into_iter()
            .rev()
            .collect::<Vec<_>>())
    } else {
        Ok(vec![])
    }
}

pub(crate) fn decode_stream(encoder_order: &[Encoding], mut stream: BodyStream) -> BodyStream {
    fn wrap_reader<R>(reader: R) -> BodyStream
    where
        R: AsyncRead + Send + 'static,
    {
        Box::pin(ReaderStream::new(reader).map_err(|e| e.to_string()))
    }

    for encoder in encoder_order {
        let reader = StreamReader::new(stream.map_err(io::Error::other));
        stream = match encoder {
            Encoding::Br => wrap_reader(BrotliDecoder::new(reader)),
            Encoding::Deflate => wrap_reader(ZlibDecoder::new(reader)),
            Encoding::Gzip => wrap_reader(GzipDecoder::new(reader)),
            Encoding::Zstd => wrap_reader(ZstdDecoder::new(reader)),
        };
    }

    stream
}

pub(crate) async fn decode(encoder_order: &[Encoding], mut data: Bytes) -> Result<Bytes, NetworkError> {
    for encoder in encoder_order {
        data = match encoder {
            Encoding::Br => decode_br(data).await?,
            Encoding::Deflate => decode_deflate(data).await?,
            Encoding::Gzip => decode_gzip(data).await?,
            Encoding::Zstd => decode_zstd(data).await?,
        };
    }

    Ok(data)
}

async fn decode_br(data: Bytes) -> Result<Bytes, NetworkError> {
    let vec_data = data.to_vec();
    let reader = BufReader::new(vec_data.as_slice());
    let mut decoder = BrotliDecoder::new(reader);

    trace!("Decoding Brotli data of length {}", data.len());

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .await
        .map_err(|e| NetworkError::DecodingError(e.to_string()))?;

    Ok(decompressed.into())
}

async fn decode_deflate(data: Bytes) -> Result<Bytes, NetworkError> {
    let vec_data = data.to_vec();
    let reader = BufReader::new(vec_data.as_slice());
    let mut decoder = ZlibDecoder::new(reader);

    trace!("Decoding Deflate data of length {}", data.len());

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .await
        .map_err(|e| NetworkError::DecodingError(e.to_string()))?;

    Ok(decompressed.into())
}

async fn decode_gzip(data: Bytes) -> Result<Bytes, NetworkError> {
    let vec_data = data.to_vec();
    let reader = BufReader::new(vec_data.as_slice());
    let mut decoder = GzipDecoder::new(reader);

    trace!("Decoding Gzip data of length {}", data.len());

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .await
        .map_err(|e| NetworkError::DecodingError(e.to_string()))?;

    Ok(decompressed.into())
}

async fn decode_zstd(data: Bytes) -> Result<Bytes, NetworkError> {
    let vec_data = data.to_vec();
    let reader = BufReader::new(vec_data.as_slice());
    let mut decoder = ZstdDecoder::new(reader);

    trace!("Decoding Zstd data of length {}", data.len());

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .await
        .map_err(|e| NetworkError::DecodingError(e.to_string()))?;

    Ok(decompressed.into())
}
