// todo: combine both senders into one
pub mod ftp_cache;
pub mod transfer_cache;

use crate::cache::{Cache, FTP_CLEANING_DELAY_S, TRANSFER_CLEANING_DELAY_S};
use crate::info;
use crate::utils::time;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref IS_CLEANUP_RUNNING: AtomicBool = AtomicBool::new(false);
}

/// Start the cache cleanup system
pub async fn cleanup() {
    let cleanup_is_running = IS_CLEANUP_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err();

    if !cleanup_is_running {
        info!("Cleanup system has been initialized");

        tokio::spawn(async move {
            let mut last_cleanup_ftp = time::now();
            let mut last_cleanup_transfer = last_cleanup_ftp;

            while { !super::SENDER_FTP.is_closed() || !super::SENDER_TRANSFER.is_closed() } {
                let now = time::now();

                if now - last_cleanup_ftp >= FTP_CLEANING_DELAY_S {
                    Cache::clean_ftp(now).await;
                    last_cleanup_ftp = now;
                }

                if now - last_cleanup_transfer >= TRANSFER_CLEANING_DELAY_S {
                    Cache::clean_transfers(now).await;
                    last_cleanup_transfer = now;
                }
            }
        });
    }
}
