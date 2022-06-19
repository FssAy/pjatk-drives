use crate::cache::cached_value::CachedValueBlocking;
use crate::cache::ftp::{FileContentPack, TransferID, TransferInfo};
use crate::cache::FtpClientID;
use crate::handler::endpoints::login::LoginData;
use ssh2::{FileStat, Sftp};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

/// Used to return a value back to the sender of the directive
pub type Callback<T> = oneshot::Sender<T>;

/// SFTP related commands
pub enum DirectiveFTP {
    SFTPConnect {
        login_data: LoginData,
        callback: Callback<anyhow::Result<FtpClientID>>,
    },
    SFTPAddClient {
        id: FtpClientID,
        stream: Sftp,
    },
    SFTPClientExists {
        id_raw: String,
        callback: Callback<bool>,
    },
    SFTPExecute {
        id: FtpClientID,
        callback: Callback<bool>,
        ftp_directive: DirectiveExecuteFTP,
    },
    Clean {
        now: i64,
    },
}

/// File Transfer related commands
pub enum DirectiveTransfer {
    GetTransferInfo {
        transfer_id: TransferID,
        callback: Callback<Option<CachedValueBlocking<TransferInfo>>>,
    },
    AddTransferInfo {
        transfer_id: TransferID,
        transfer_info: CachedValueBlocking<TransferInfo>,
    },
    RemoveTransferInfo {
        transfer_id: TransferID,
    },
    Clean {
        now: i64,
    },
}

/// Commands representing what ftp operation to perform
pub enum DirectiveExecuteFTP {
    ReadDir {
        dir: String,
        callback: Callback<anyhow::Result<Vec<(PathBuf, FileStat)>>>,
    },
    #[deprecated]
    ReadFile {
        file: String,
        callback: Callback<
            anyhow::Result<mpsc::Sender<(usize, oneshot::Sender<anyhow::Result<Vec<u8>>>)>>,
        >,
    },
    TransferFile {
        transfer_id: TransferID,
        filename: Option<String>,
        callback: Callback<anyhow::Result<FileContentPack>>,
    },
}
