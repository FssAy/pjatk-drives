use super::super::*;

/// Used to simplify sending directives to the cache
pub struct TransferSender<T> {
    inner: mpsc::Sender<DirectiveTransfer>,
    callback: Option<Callback<T>>,
    rx: Option<oneshot::Receiver<T>>,
}

#[allow(dead_code)]
impl<T> TransferSender<T> {
    /// Creates new instance and opens the oneshot channel
    ///
    /// Used only for sending directives with callback.
    /// The non-callback directives doesn't require oneshot channel to be opened
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel::<T>();
        Self {
            inner: SENDER_TRANSFER.clone(),
            callback: Some(tx),
            rx: Some(rx),
        }
    }

    /// Panics if callback has been already taken
    pub fn take_callback(&mut self) -> Callback<T> {
        self.callback.take().unwrap()
    }

    /// Send directive with the callback
    pub async fn send_with_callback(self, directive: DirectiveTransfer) -> T {
        let _ = self.inner.send(directive).await;
        self.rx.unwrap().await.unwrap()
    }

    /// Send directive with the callback without asynchronous operations
    pub fn send_with_callback_blocking(self, directive: DirectiveTransfer) -> T {
        let _ = self.inner.blocking_send(directive);
        self.rx.unwrap().blocking_recv().unwrap()
    }

    /// Send directive without the callback
    pub async fn send(directive: DirectiveTransfer) {
        SENDER_TRANSFER.send(directive).await.ok();
    }

    /// Send directive without the callback and asynchronous operations
    pub fn send_blocking(directive: DirectiveTransfer) {
        SENDER_TRANSFER.blocking_send(directive).ok();
    }
}

/// All the transfer cache related directives are processed here
pub async fn handler(mut rx: Receiver<DirectiveTransfer>) {
    let mut transfers = HashMap::<TransferID, CachedValueBlocking<TransferInfo>>::new();

    while let Some(directive) = rx.recv().await {
        match directive {
            DirectiveTransfer::GetTransferInfo {
                transfer_id,
                callback,
            } => {
                callback
                    .send(transfers.get(&transfer_id).map(Clone::clone))
                    .ok();
            }

            DirectiveTransfer::AddTransferInfo {
                transfer_id,
                transfer_info,
            } => {
                transfers.insert(transfer_id, transfer_info);
            }

            DirectiveTransfer::RemoveTransferInfo { transfer_id } => {
                transfers.remove(&transfer_id);
            }

            DirectiveTransfer::Clean { now } => {
                let t_vec: Vec<(TransferID, CachedValueBlocking<TransferInfo>)> = transfers
                    .iter()
                    .map(|(id, info)| (id.clone(), info.clone()))
                    .collect();

                tokio::task::spawn_blocking(move || {
                    for (id, info) in t_vec {
                        if info.read().last_usage_timestamp >= TRANSFER_LIFETIME_S {
                            Cache::remove_transfer_blocking(id);
                        }
                    }
                });
            }
        }
    }
}
