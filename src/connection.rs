use std::io;
use std::task::Poll;

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub(crate) use std::os::unix::io::AsRawFd as AsRawFdOrSocket;
    } else if #[cfg(windows)] {
        pub(crate) use std::os::windows::io::AsRawSocket as AsRawFdOrSocket;
    } else {
        compile_error!("async-imap-lite does not support this target OS");
    }
}

pub(crate) use async_io::Async;
use async_stream_packed::SyncableWithWakerAsyncStream;
pub(crate) use async_stream_tls_upgrader::{
    TlsClientUpgrader, UpgradableAsyncStream, UpgraderExtRefer,
};
use futures_lite::future;
pub(crate) use futures_lite::{AsyncRead, AsyncWrite};

use crate::imap::{Connection, Error, Result};
use crate::util::optimistic;

pub struct AsyncConnection<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    connection: Connection<SyncableWithWakerAsyncStream<UpgradableAsyncStream<Async<AS>, ASTU>>>,
}

impl<AS, ASTU> AsyncConnection<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite + Unpin,
    ASTU: TlsClientUpgrader<Async<AS>> + UpgraderExtRefer<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new(
        stream: UpgradableAsyncStream<Async<AS>, ASTU>,
        debug: bool,
        greeting_read: bool,
    ) -> Self {
        let mut stream = Some(stream);
        let connection = future::poll_fn(|cx| {
            let stream =
                SyncableWithWakerAsyncStream::new(stream.take().expect("never"), cx.waker());
            let connection = Connection::new(stream, debug, greeting_read);
            Poll::Ready(connection)
        })
        .await;

        Self { connection }
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1185-L1193
    pub async fn read_greeting(&mut self) -> Result<Vec<u8>> {
        assert!(
            !self.connection.greeting_read,
            "Greeting can only be read once"
        );

        let mut v = Vec::new();
        self.readline(&mut v).await?;
        self.connection.greeting_read = true;

        Ok(v)
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1157-L1159
    pub async fn run_command_and_check_ok<S: AsRef<str>>(&mut self, command: S) -> Result<()> {
        self.run_command_and_read_response(command)
            .await
            .map(|_| ())
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1172-L1178
    pub async fn run_command_and_read_response<S: AsRef<str>>(
        &mut self,
        untagged_command: S,
    ) -> Result<Vec<u8>> {
        self.run_command(untagged_command.as_ref()).await?;

        let mut v = Vec::new();
        self.read_response_onto(&mut v).await?;

        Ok(v)
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1287
    async fn readline(&mut self, into: &mut Vec<u8>) -> Result<usize> {
        loop {
            match self.connection.readline(into) {
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
                ret => break ret,
            }
            optimistic(
                self.connection
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .readable(),
            )
            .await?;
        }
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1199
    async fn run_command(&mut self, untagged_command: &str) -> Result<()> {
        loop {
            match self.connection.run_command(untagged_command) {
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
                ret => break ret,
            }
            optimistic(
                self.connection
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .writable(),
            )
            .await?;
        }
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L1215
    async fn read_response_onto(&mut self, data: &mut Vec<u8>) -> Result<()> {
        loop {
            match self.connection.read_response_onto(data) {
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
                ret => break ret,
            }
            optimistic(
                self.connection
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .get_mut()
                    .readable(),
            )
            .await?;
        }
    }
}
