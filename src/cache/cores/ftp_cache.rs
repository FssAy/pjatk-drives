use super::super::*;
use crate::utils::time;

pub struct FtpSender<T> {
    inner: mpsc::Sender<DirectiveFTP>,
    callback: Option<Callback<T>>,
    rx: Option<oneshot::Receiver<T>>,
}

impl<T> FtpSender<T> {
    /// Creates new instance and opens the oneshot channel
    ///
    /// Used only for sending directives with callback.
    /// The non-callback directives doesn't require oneshot channel to be opened
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel::<T>();
        Self {
            inner: SENDER_FTP.clone(),
            callback: Some(tx),
            rx: Some(rx),
        }
    }

    /// Panics if callback has been already taken
    pub fn take_callback(&mut self) -> Callback<T> {
        self.callback.take().unwrap()
    }

    /// Send directive with the callback
    pub async fn send_with_callback(self, directive: DirectiveFTP) -> T {
        let _ = self.inner.send(directive).await;
        self.rx.unwrap().await.unwrap()
    }

    /// Send directive with the callback without asynchronous operations
    pub fn send_with_callback_blocking(self, directive: DirectiveFTP) -> T {
        let _ = self.inner.blocking_send(directive);
        self.rx.unwrap().blocking_recv().unwrap()
    }

    /// Send directive without the callback
    pub async fn send(directive: DirectiveFTP) {
        SENDER_FTP.send(directive).await.ok();
    }

    /// Send directive without the callback and asynchronous operations
    pub fn send_blocking(directive: DirectiveFTP) {
        SENDER_FTP.blocking_send(directive).ok();
    }
}

pub struct Resources {
    in_usage: bool,
    last_usage_timestamp: i64,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            in_usage: false,
            last_usage_timestamp: time::now(),
        }
    }
}

/// All the ftp cache related directives are processed here
pub async fn handler(mut rx: Receiver<DirectiveFTP>) {
    let mut ftp_clients = HashMap::<FtpClientID, Arc<Sftp>>::new();
    let mut ftp_resources = HashMap::<FtpClientID, Resources>::new();

    while let Some(directive) = rx.recv().await {
        match directive {
            // When Login endpoint called
            // It tries to find the already existing client, or if failed, create a new one
            DirectiveFTP::SFTPConnect {
                login_data,
                callback,
            } => {
                let id = ftp::hash_login(&login_data);

                if ftp_clients.contains_key(&id) {
                    // update ftp client usage
                    if let Some(resources) = ftp_resources.get_mut(&id) {
                        resources.last_usage_timestamp = time::now();
                    }

                    callback.send(Ok(id)).ok();
                } else {
                    tokio::task::spawn_blocking(move || {
                        callback
                            .send(
                                ftp::connect(login_data)
                                    .map(|ftp| Cache::add_ftp_client_blocking(id, ftp)),
                            )
                            .ok();
                    });
                }
            }

            // Private directive
            // Adds the client to the map
            DirectiveFTP::SFTPAddClient { id, stream } => {
                ftp_clients.insert(id.clone(), Arc::new(stream));
                ftp_resources.insert(id, Resources::default());
            }

            // Check if an sftp client exists
            DirectiveFTP::SFTPClientExists { id_raw, callback } => {
                callback.send(ftp_clients.contains_key(&id_raw)).ok();
            }

            // Responsible for finding the client and executing certain ftp commands
            DirectiveFTP::SFTPExecute {
                id,
                callback,
                ftp_directive,
            } => {
                // get ftp client
                let stream = if let Some(stream) = ftp_clients.get(&id) {
                    stream.clone()
                } else {
                    callback.send(false).ok();
                    continue;
                };

                // update usage timestamp for the client
                if let Some(resources) = ftp_resources.get_mut(&id) {
                    resources.last_usage_timestamp = time::now();
                    callback.send(true).ok();
                } else {
                    ftp_clients.remove(&id);
                    callback.send(false).ok();
                    continue;
                }

                // spawn task responsible for executing ftp command
                tokio::task::spawn_blocking(move || {
                    match ftp_directive {
                        // List all entities in the dir
                        DirectiveExecuteFTP::ReadDir { dir, callback } => {
                            callback
                                .send(stream.readdir(dir.as_ref()).map_err(|e| e.into()))
                                .ok();
                        }

                        // Stream file content [deprecated]
                        DirectiveExecuteFTP::ReadFile { file, callback } => {
                            // try to open the file
                            let mut file = match stream.open(file.as_ref()) {
                                Ok(file) => file,
                                Err(error) => {
                                    callback.send(Err(error.into())).ok();
                                    return;
                                }
                            };

                            // create an mpsc channel for streaming file bytes and return the sender in the callback
                            let (file_tx, mut file_rx) = mpsc::channel::<(
                                usize,
                                oneshot::Sender<anyhow::Result<Vec<u8>>>,
                            )>(32);
                            callback.send(Ok(file_tx)).ok();

                            // stream bytes of the file
                            while let Some((buffer_size, callback)) = file_rx.blocking_recv() {
                                let mut buffer = vec![0u8; buffer_size];
                                match file.read(&mut buffer) {
                                    Ok(read_size) => {
                                        buffer.drain(read_size..);
                                        if callback.send(Ok(buffer)).is_err() {
                                            return;
                                        };

                                        if read_size == 0 {
                                            return;
                                        }
                                    }
                                    Err(error) => {
                                        callback.send(Err(error.into())).ok();
                                        return;
                                    }
                                }
                            }

                            // cleanup
                            file_rx.close();
                            drop(file);
                        }

                        // Start or continue the file transfer
                        DirectiveExecuteFTP::TransferFile {
                            transfer_id,
                            filename,
                            callback,
                        } => {
                            let transfer_info = if let Some(transfer_info) =
                                Cache::get_transfer_info_blocking(transfer_id.clone())
                            {
                                transfer_info
                            } else {
                                if filename.is_none() {
                                    callback
                                        .send(Err(anyhow::Error::msg(
                                            "new transfer without filename",
                                        )))
                                        .ok();
                                    return;
                                }

                                let filename = unsafe { filename.unwrap_unchecked() };
                                let mut file = match stream.open(filename.as_ref()) {
                                    Ok(file) => file,
                                    Err(error) => {
                                        callback.send(Err(error.into())).ok();
                                        return;
                                    }
                                };

                                let mut transfer_info = TransferInfo::default();

                                match file.stat() {
                                    Ok(stat) => {
                                        let size = stat.size.unwrap_or(0);
                                        if size == 0 {
                                            // do not download files with size of 0
                                            callback
                                                .send(Err(anyhow::Error::msg(
                                                    "invalid file size 0",
                                                )))
                                                .ok();
                                            return;
                                        }
                                        transfer_info.file_size = size;
                                    }
                                    Err(error) => {
                                        callback.send(Err(error.into())).ok();
                                        return;
                                    }
                                }

                                transfer_info.file = Some(file);
                                CachedValueBlocking::new(transfer_info)
                            };

                            let mut transfer_info_guard = transfer_info.write();

                            let read_file =
                                |guard: &mut RwLockWriteGuard<TransferInfo>,
                                 buffer: &mut Vec<u8>| {
                                    let file = guard.file.as_mut().unwrap();
                                    file.read(buffer)
                                };

                            let mut buffer = vec![0u8; transfer_info_guard.chunk_size];
                            match read_file(&mut transfer_info_guard, &mut buffer) {
                                Ok(read_size) => {
                                    buffer.drain(read_size..);
                                    transfer_info_guard.total_read_size += read_size as u64;

                                    let packs_left = (transfer_info_guard.file_size
                                        / transfer_info_guard.chunk_size as u64)
                                        - transfer_info_guard.chunks_sent as u64;
                                    let content_pack = FileContentPack {
                                        no: transfer_info_guard.chunks_sent as u64,
                                        is_last: packs_left == 0 || buffer.is_empty(),
                                        size: buffer.len() as u64,
                                        bytes: buffer,
                                        packs_left,
                                    };

                                    callback.send(Ok(content_pack)).ok();
                                }
                                Err(error) => {
                                    callback.send(Err(error.into())).ok();
                                    return;
                                }
                            }

                            transfer_info_guard.chunks_sent += 1;
                            let chunks_sent = transfer_info_guard.chunks_sent;
                            transfer_info_guard.last_usage_timestamp = time::now();

                            drop(transfer_info_guard);
                            if chunks_sent == 1 {
                                Cache::add_transfer_info_blocking(transfer_id, transfer_info);
                            }
                        }
                    }
                });
            }

            DirectiveFTP::Clean { now } => {
                ftp_resources.retain(|id, resources| {
                    if resources.in_usage {
                        return true;
                    }

                    if now - resources.last_usage_timestamp >= FTP_CLIENT_LIFETIME_S {
                        debug!("Removing ftp client ({id})");
                        ftp_clients.remove(id);
                        return false;
                    }

                    true
                });
            }
        }

        // todo: implement cleanup code
        /*
                        // let now = chrono::Local::now().timestamp();
                        //
                        // if now - cleaning_timestamp >= CLEANING_DELAY_S {
                        //     let cleanup_span = span!(Level::DEBUG, "Cleanup");
                        //     let _ = cleanup_span.enter();
                        //
                        //     ftp_resources.retain(|id, resources| {
                        //         if resources.in_usage {
                        //             return true;
                        //         }
                        //
                        //         if now - resources.last_usage_timestamp >= FTP_CLIENT_LIFETIME_S {
                        //             debug!("Removing ftp client ({id})");
                        //             ftp_clients.remove(id);
                        //             return false;
                        //         }
                        //
                        //         true
                        //     });
                        //     cleaning_timestamp = now;
                        // }

        */
    }
}
