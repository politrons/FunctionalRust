use serde::{Serialize, Deserialize};
use serde_cbor;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: u32,
    pub content: String,
    pub timestamp: u64,
}

pub fn serialize_and_compress(message: &Message) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Serialize the message to CBOR format
    let serialized = serde_cbor::to_vec(message)?;

    // Compress the serialized data
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;

    Ok(compressed)
}

pub fn decompress_and_deserialize(data: &[u8]) -> Result<Message, Box<dyn std::error::Error>> {
    // Decompress the data
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    // Deserialize the message from CBOR format
    let message = serde_cbor::from_slice(&decompressed_data)?;

    Ok(message)
}
