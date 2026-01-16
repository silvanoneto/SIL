//! I/O intrinsics for JSIL format
//!
//! These functions bridge LIS code with Rust implementations
//! for reading/writing JSIL files.

use crate::error::{Error, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sil_core::state::SilState;
use sil_core::io::jsil::{JsilReader, JsilWriter, JsilCompressor, CompressionMode};
use sil_core::io::jsonl::JsonlRecord;
use std::path::Path;

/// Intrinsic function definitions for JSIL I/O
pub struct JsilIntrinsics;

impl JsilIntrinsics {
    /// @sil_io fn read_jsil(path: String) -> State
    ///
    /// Reads a JSIL file and deserializes the first state found.
    pub fn read_jsil(path: &str) -> Result<SilState> {
        let mut reader = JsilReader::load(Path::new(path)).map_err(|e| Error::IoError {
            message: format!("Failed to load JSIL file: {}", e),
        })?;

        // Search for first data record with state
        while let Some(record) = reader
            .next_record::<JsonlRecord>()
            .map_err(|e| Error::IoError {
                message: format!("Failed to read JSIL record: {}", e),
            })?
        {
            if let JsonlRecord::Data { bytes, .. } = record {
                let decoded = STANDARD.decode(&bytes).map_err(|e| Error::IoError {
                    message: format!("Failed to decode base64: {}", e),
                })?;

                if decoded.len() != 16 {
                    return Err(Error::IoError {
                        message: format!("Invalid state data: expected 16 bytes, got {}", decoded.len()),
                    });
                }

                let mut array = [0u8; 16];
                array.copy_from_slice(&decoded);
                return Ok(SilState::from_bytes(&array));
            }
        }

        Err(Error::IoError {
            message: "No state data found in JSIL file".into(),
        })
    }

    /// @sil_io fn write_jsil(path: String, state: State)
    ///
    /// Writes a SilState to a JSIL file with default compression (XorRotate).
    pub fn write_jsil(path: &str, state: &SilState) -> Result<()> {
        Self::write_jsil_with_mode(path, state, CompressionMode::XorRotate)
    }

    /// @sil_io fn write_jsil_with_mode(path: String, state: State, mode: CompressionMode)
    ///
    /// Writes a SilState to a JSIL file with specified compression mode.
    pub fn write_jsil_with_mode(
        path: &str,
        state: &SilState,
        mode: CompressionMode,
    ) -> Result<()> {
        let compressor = JsilCompressor::new(mode, 0x5A);
        let mut writer = JsilWriter::new(compressor);

        // Create data record with serialized state
        let state_bytes = state.to_bytes();
        let record = JsonlRecord::Data {
            offset: 0,
            len: 16, // 16 layers
            bytes: STANDARD.encode(&state_bytes[..]),
        };

        writer.write_record(&record).map_err(|e| Error::IoError {
            message: format!("Failed to write JSIL record: {}", e),
        })?;

        writer.save(Path::new(path)).map_err(|e| Error::IoError {
            message: format!("Failed to save JSIL file: {}", e),
        })?;

        Ok(())
    }

    /// @sil_io fn stream_jsil(state: State) -> Vec<u8>
    ///
    /// Serializes a SilState to JSIL format in memory (for network transmission).
    pub fn stream_jsil(state: &SilState) -> Result<Vec<u8>> {
        let compressor = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);
        let mut writer = JsilWriter::new(compressor);

        let state_bytes = state.to_bytes();
        let record = JsonlRecord::Data {
            offset: 0,
            len: 16,
            bytes: STANDARD.encode(&state_bytes[..]),
        };

        writer.write_record(&record).map_err(|e| Error::IoError {
            message: format!("Failed to write stream record: {}", e),
        })?;

        // Get bytes by finalizing to a vector
        let mut buffer = Vec::new();
        writer.finalize(&mut buffer).map_err(|e| Error::IoError {
            message: format!("Failed to finalize stream: {}", e),
        })?;

        Ok(buffer)
    }

    /// @sil_io fn receive_jsil(data: &[u8]) -> State
    ///
    /// Deserializes a SilState from JSIL bytes (received from network).
    pub fn receive_jsil(data: &[u8]) -> Result<SilState> {
        // Write to temp file first (JsilReader only supports files)
        use std::io::Write;
        let temp_path = "/tmp/lis_receive_jsil.tmp";
        let mut temp_file = std::fs::File::create(temp_path).map_err(|e| Error::IoError {
            message: format!("Failed to create temp file: {}", e),
        })?;
        temp_file.write_all(data).map_err(|e| Error::IoError {
            message: format!("Failed to write temp file: {}", e),
        })?;
        drop(temp_file);

        let mut reader = JsilReader::load(temp_path).map_err(|e| Error::IoError {
            message: format!("Failed to load JSIL from bytes: {}", e),
        })?;

        while let Some(record) = reader
            .next_record::<JsonlRecord>()
            .map_err(|e| Error::IoError {
                message: format!("Failed to read JSIL record: {}", e),
            })?
        {
            if let JsonlRecord::Data { bytes, .. } = record {
                let decoded = STANDARD.decode(&bytes).map_err(|e| Error::IoError {
                    message: format!("Failed to decode base64: {}", e),
                })?;

                if decoded.len() != 16 {
                    return Err(Error::IoError {
                        message: format!("Invalid state data: expected 16 bytes, got {}", decoded.len()),
                    });
                }

                let mut array = [0u8; 16];
                array.copy_from_slice(&decoded);

                // Cleanup temp file
                let _ = std::fs::remove_file(temp_path);

                return Ok(SilState::from_bytes(&array));
            }
        }

        // Cleanup temp file
        let _ = std::fs::remove_file(temp_path);

        Err(Error::IoError {
            message: "No state data in stream".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_read_roundtrip() {
        let state = SilState::neutral();
        let path = "/tmp/test_stdlib_state.jsil";

        // Write
        JsilIntrinsics::write_jsil(path, &state).unwrap();

        // Read
        let loaded = JsilIntrinsics::read_jsil(path).unwrap();

        assert_eq!(state, loaded);

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_stream_receive_roundtrip() {
        let state = SilState::neutral();

        // Stream
        let data = JsilIntrinsics::stream_jsil(&state).unwrap();

        assert!(!data.is_empty(), "Streamed data should not be empty");

        // Receive
        let received = JsilIntrinsics::receive_jsil(&data).unwrap();

        assert_eq!(state, received);
    }

    #[test]
    fn test_write_with_different_modes() {
        let state = SilState::neutral();

        for mode in [
            CompressionMode::None,
            CompressionMode::Xor,
            CompressionMode::XorRotate,
        ] {
            let path = format!("/tmp/test_mode_{:?}.jsil", mode);

            // Write with specific mode
            JsilIntrinsics::write_jsil_with_mode(&path, &state, mode).unwrap();

            // Read back
            let loaded = JsilIntrinsics::read_jsil(&path).unwrap();

            assert_eq!(state, loaded, "Roundtrip failed for mode {:?}", mode);

            // Cleanup
            let _ = fs::remove_file(&path);
        }
    }
}
