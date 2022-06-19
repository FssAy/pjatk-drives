use super::*;
use ssh2::File;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub enum NoCallback {}

/// Used in transfer cache system for saving metadata about the file transfer.
///
/// It also contains the file handle to save progress of the transfer
pub struct TransferInfo {
    pub last_usage_timestamp: i64,
    pub file_size: u64,
    pub total_read_size: u64,
    pub chunk_size: usize,
    pub chunks_sent: usize,
    pub file: Option<File>,
}

impl Default for TransferInfo {
    fn default() -> Self {
        Self {
            last_usage_timestamp: chrono::Local::now().timestamp(),
            file_size: 0,
            total_read_size: 0,
            chunk_size: 1024,
            chunks_sent: 0,
            file: None,
        }
    }
}

impl TransferInfo {
    #[allow(dead_code)]
    #[deprecated]
    /// Used to calculate optimal mpsc channel buffer based on the file and chunk size
    pub fn calculate_mpsc_buffer(&self) -> usize {
        let buffer = self.file_size / self.chunk_size as u64;
        if buffer > 32 {
            32
        } else {
            buffer as usize
        }
    }
}

/*pub struct TransferDirective {
    pub buffer_size: usize,
    pub callback: oneshot::Sender<anyhow::Result<Vec<u8>>>
}*/

/// Container of the successful file read operation result.
pub struct FileContentPack {
    /// bytes read in the single operation
    pub bytes: Vec<u8>,

    /// number of the pack
    pub no: u64,

    /// is this pack the last one to contain data
    pub is_last: bool,

    /// how many bytes have been read in the single operation
    pub size: u64,

    /// how many packs left to read
    pub packs_left: u64,
}

/// Identification of the user-file specific transfer.
/// Never revealed to the user
#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub struct TransferID {
    inner: Arc<String>,
}

impl TransferID {
    pub fn new(id: FtpClientID, filename: impl ToString) -> Self {
        Self {
            inner: Arc::new(format!("{id}-{}", filename.to_string())),
        }
    }
}
