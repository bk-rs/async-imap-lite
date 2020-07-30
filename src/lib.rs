use std::io;
use std::net::TcpStream;

use async_io::Async;
use async_net::AsyncToSocketAddrs;
pub use async_stream_tls_upgrader::{TlsClientUpgrader, Upgrader};
use imap_patch_for_async_imap_lite as imap;

#[cfg(feature = "async_native_tls")]
pub use async_stream_tls_upgrader::AsyncNativeTlsClientTlsUpgrader;
#[cfg(feature = "async_tls")]
pub use async_stream_tls_upgrader::AsyncTlsClientTlsUpgrader;

use crate::connection::{AsyncRead, AsyncWrite, UpgradableAsyncStream, UpgraderExtRefer};
use crate::imap::{Error, Result};

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L137-L155
pub async fn connect_async<A: AsyncToSocketAddrs, ASTU>(
    addr: A,
    tls_upgrader: ASTU,
    debug: Option<bool>,
) -> Result<AsyncClient<TcpStream, ASTU>>
where
    ASTU: TlsClientUpgrader<Async<TcpStream>> + UpgraderExtRefer<Async<TcpStream>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    let addr = addr
        .to_socket_addrs()
        .await?
        .next()
        .ok_or(Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid addr",
        )))?;

    let stream = Async::<TcpStream>::connect(addr).await?;
    let mut stream = UpgradableAsyncStream::new(stream, tls_upgrader);
    stream.upgrade().await?;

    let conn = AsyncConnection::new(stream, debug.unwrap_or(false), false).await;
    let mut client = AsyncClient::new(conn);

    client.read_greeting().await?;

    Ok(client)
}

//
//
//
mod client;
mod connection;
mod session;
mod util;

pub use client::AsyncClient;
pub use connection::AsyncConnection;
pub use session::AsyncSession;
