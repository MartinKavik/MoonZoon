use crate::helper::ReadToVec;
use anyhow::{Context, Result};
use apply::Also;
use async_trait::async_trait;
use brotli::{enc::backward_references::BrotliEncoderParams, CompressorReader as BrotliEncoder};
use flate2::{bufread::GzEncoder, Compression as GzCompression};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, io::AsyncWriteExt, task::spawn_blocking};

#[async_trait]
pub trait FileCompressor {
    async fn compress_file(content: Arc<Vec<u8>>, path: &Path, extension: &str) -> Result<()> {
        let path = compressed_file_path(path, extension);
        let mut file_writer = fs::File::create(&path)
            .await
            .with_context(|| format!("Failed to create the file {:#?}", path))?;

        let compressed_content = spawn_blocking(move || Self::compress(&content)).await??;

        file_writer.write_all(&compressed_content).await?;
        file_writer.flush().await?;
        Ok(())
    }

    fn compress(bytes: &[u8]) -> Result<Vec<u8>>;
}

fn compressed_file_path(path: &Path, extension: &str) -> PathBuf {
    let new_extension = path
        .extension()
        .unwrap_or_default()
        .to_owned()
        .also(|old_extension| old_extension.push(format!(".{}", extension)));
    path.with_extension(new_extension)
}

// ------ Brotli ------

pub struct BrotliFileCompressor;

#[async_trait]
impl FileCompressor for BrotliFileCompressor {
    fn compress(bytes: &[u8]) -> Result<Vec<u8>> {
        BrotliEncoder::with_params(bytes, 0, &BrotliEncoderParams::default()).read_to_vec()
    }
}

// ------ Gzip ------

pub struct GzipFileCompressor;

#[async_trait]
impl FileCompressor for GzipFileCompressor {
    fn compress(bytes: &[u8]) -> Result<Vec<u8>> {
        GzEncoder::new(bytes, GzCompression::best()).read_to_vec()
    }
}
