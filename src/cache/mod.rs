// todo: external library

mod cached_value;
mod cores;
mod directives;
pub mod ftp;

use crate::cache::cores::ftp_cache::FtpSender;
use crate::cache::cores::transfer_cache::TransferSender;
use crate::cache::ftp::{FileContentPack, NoCallback, TransferID, TransferInfo};
use crate::handler::endpoints::login::LoginData;

use crate::logging::*;
use cached_value::*;
use directives::*;

use sha2::Digest;
use ssh2::{FileStat, Sftp};

use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Read;
use std::path::PathBuf;

use std::sync::{Arc, RwLockWriteGuard};

use tokio::spawn;

use crate::cache::cores::cleanup;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;

const FTP_CACHE_CAPACITY: usize = 256;
const FTP_CLEANING_DELAY_S: i64 = 60;
const FTP_CLIENT_LIFETIME_S: i64 = 60 * 5;

const TRANSFER_CACHE_CAPACITY: usize = 1024;
const TRANSFER_CLEANING_DELAY_S: i64 = 40;
const TRANSFER_LIFETIME_S: i64 = 30;

pub type FtpClientID = Arc<String>;

lazy_static! {
    static ref SENDER_FTP: Sender<DirectiveFTP> = {
        info!("FTP cache has been initialized");
        let (tx, rx) = mpsc::channel(FTP_CACHE_CAPACITY);
        spawn(cores::ftp_cache::handler(rx));
        tx
    };
    static ref SENDER_TRANSFER: Sender<DirectiveTransfer> = {
        info!("Transfer cache has been initialized");
        let (tx, rx) = mpsc::channel(TRANSFER_CACHE_CAPACITY);
        spawn(cores::transfer_cache::handler(rx));
        tx
    };
}

/// Collection of functions to perform specific operations on the cache system
pub struct Cache;

impl Cache {
    /// Makes sure that the cache system is running
    ///
    /// Cache system can work without calling this function,
    /// but the cleanup system won't start.
    pub async fn init() {
        let _ = SENDER_FTP.clone();
        let _ = SENDER_TRANSFER.clone();
        cleanup().await;
    }

    fn add_ftp_client_blocking(id: FtpClientID, stream: Sftp) -> FtpClientID {
        let _ = SENDER_FTP.blocking_send(DirectiveFTP::SFTPAddClient {
            id: id.clone(),
            stream,
        });

        id
    }

    async fn get_transfer_info(
        transfer_id: TransferID,
    ) -> Option<CachedValueBlocking<TransferInfo>> {
        let mut sender = TransferSender::new();
        let directive = DirectiveTransfer::GetTransferInfo {
            transfer_id,
            callback: sender.take_callback(),
        };
        sender.send_with_callback(directive).await
    }

    fn get_transfer_info_blocking(
        transfer_id: TransferID,
    ) -> Option<CachedValueBlocking<TransferInfo>> {
        let mut sender = TransferSender::new();
        let directive = DirectiveTransfer::GetTransferInfo {
            transfer_id,
            callback: sender.take_callback(),
        };
        sender.send_with_callback_blocking(directive)
    }

    fn add_transfer_info_blocking(
        transfer_id: TransferID,
        transfer_info: CachedValueBlocking<TransferInfo>,
    ) {
        TransferSender::<NoCallback>::send_blocking(DirectiveTransfer::AddTransferInfo {
            transfer_id,
            transfer_info,
        });
    }

    fn remove_transfer_blocking(transfer_id: TransferID) {
        TransferSender::<NoCallback>::send_blocking(DirectiveTransfer::RemoveTransferInfo {
            transfer_id,
        });
    }

    pub async fn clean_ftp(now: i64) {
        FtpSender::<NoCallback>::send(DirectiveFTP::Clean { now }).await;
    }

    pub async fn clean_transfers(now: i64) {
        TransferSender::<NoCallback>::send(DirectiveTransfer::Clean { now }).await;
    }

    pub async fn ftp_client_exists(id: impl ToString) -> bool {
        let mut sender = FtpSender::new();
        let directive = DirectiveFTP::SFTPClientExists {
            id_raw: id.to_string(),
            callback: sender.take_callback(),
        };
        sender.send_with_callback(directive).await
    }

    pub async fn ftp_connect(login_data: LoginData) -> anyhow::Result<FtpClientID> {
        let mut sender = FtpSender::new();
        let directive = DirectiveFTP::SFTPConnect {
            login_data,
            callback: sender.take_callback(),
        };
        sender.send_with_callback(directive).await
    }

    pub async fn ftp_read_dir(
        id: FtpClientID,
        dir: impl ToString,
    ) -> anyhow::Result<Vec<(PathBuf, FileStat)>> {
        let (tx_check, rx_check) = oneshot::channel();
        let (tx, rx) = oneshot::channel();

        let _ = SENDER_FTP
            .send(DirectiveFTP::SFTPExecute {
                id,
                callback: tx_check,
                ftp_directive: DirectiveExecuteFTP::ReadDir {
                    dir: dir.to_string(),
                    callback: tx,
                },
            })
            .await;

        if rx_check.await.unwrap() {
            rx.await.unwrap()
        } else {
            Err(anyhow::Error::msg("invalid ftp client id"))
        }
    }

    pub async fn ftp_read_file(
        id: FtpClientID,
        file: impl ToString,
    ) -> anyhow::Result<FileContentPack> {
        let (tx_check, rx_check) = oneshot::channel();
        let (tx, rx) = oneshot::channel();

        let filename = file.to_string();
        let transfer_id = TransferID::new(id.clone(), &filename);

        let _ = SENDER_FTP
            .send(DirectiveFTP::SFTPExecute {
                id,
                callback: tx_check,
                ftp_directive: DirectiveExecuteFTP::TransferFile {
                    transfer_id: transfer_id.clone(),
                    filename: Some(filename),
                    callback: tx,
                },
            })
            .await;

        if rx_check.await.unwrap() {
            if let Ok(content_pack) = rx.await {
                let content_pack = content_pack?;

                if content_pack.is_last {
                    SENDER_TRANSFER
                        .send(DirectiveTransfer::RemoveTransferInfo { transfer_id })
                        .await
                        .ok();
                }

                Ok(content_pack)
            } else {
                SENDER_TRANSFER
                    .send(DirectiveTransfer::RemoveTransferInfo { transfer_id })
                    .await
                    .ok();
                Err(anyhow::Error::msg("transfer channel has been closed"))
            }
        } else {
            Err(anyhow::Error::msg("invalid ftp client id"))
        }
    }

    #[deprecated]
    pub async fn ftp_read_file_whole(
        id: FtpClientID,
        file: impl ToString,
    ) -> anyhow::Result<Vec<u8>> {
        let (tx_check, rx_check) = oneshot::channel();
        let (tx, rx) = oneshot::channel();

        let _ = SENDER_FTP
            .send(DirectiveFTP::SFTPExecute {
                id,
                callback: tx_check,
                ftp_directive: DirectiveExecuteFTP::ReadFile {
                    file: file.to_string(),
                    callback: tx,
                },
            })
            .await;

        if rx_check.await.unwrap() {
            let buffer_size = 1024;
            let file_tx = rx.await.unwrap()?;
            let mut buffer = Vec::with_capacity(buffer_size);
            loop {
                let (bytes_tx, bytes_rx) = oneshot::channel();
                file_tx.send((buffer_size, bytes_tx)).await.ok();
                match bytes_rx.await {
                    Ok(data) => {
                        buffer.extend(data?.into_iter());
                    }
                    Err(_) => break,
                }
            }
            Ok(buffer)
        } else {
            Err(anyhow::Error::msg("invalid ftp client id"))
        }
    }
}
