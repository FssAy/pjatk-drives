mod transfer;

use crate::cache::FtpClientID;
use crate::handler::endpoints::login::LoginData;
use crate::CONFIG;
use base64::{CharacterSet, Config};
use sha2::{Digest, Sha256};
use ssh2::Sftp;

pub use transfer::*;

/// Hashes user credentials with SHA256 algorithm and then encodes it into base64
pub fn hash_login(login_data: &LoginData) -> FtpClientID {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", login_data.user, login_data.password).as_bytes());
    let id = FtpClientID::new(base64::encode_config(
        hasher.finalize().to_vec(),
        Config::new(CharacterSet::Crypt, false),
    ));
    id
}

/// Connects to the SFTP server and returns the session
pub fn connect(login_data: LoginData) -> anyhow::Result<Sftp> {
    let stream = std::net::TcpStream::connect(&CONFIG.ftp_host)?;
    let mut session = ssh2::Session::new()?;

    session.set_tcp_stream(stream);
    session.handshake()?;

    session.userauth_password(&login_data.user, &login_data.password)?;

    Ok(session.sftp()?)
}
