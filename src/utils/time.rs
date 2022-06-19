// todo: this functions runs in a while loop, use more optimized method to get the timestamp
/// Returns current timestamp.
/// Used for measuring passing time, not ideal for date operations.
pub fn now() -> i64 {
    chrono::Local::now().timestamp()
}
