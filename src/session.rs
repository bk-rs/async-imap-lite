use std::ops::{Deref, DerefMut};
use std::sync::mpsc;

use crate::connection::{
    AsRawFdOrSocket, Async, AsyncConnection, AsyncRead, AsyncWrite, TlsClientUpgrader,
    UpgraderExtRefer,
};
use crate::imap::{parse::*, types::*, validate_str, Result};

pub struct AsyncSession<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    conn: AsyncConnection<AS, ASTU>,
    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L53
    unsolicited_responses_tx: mpsc::Sender<UnsolicitedResponse>,
    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L57
    pub unsolicited_responses: mpsc::Receiver<UnsolicitedResponse>,
}

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L102-L108
impl<AS, ASTU> Deref for AsyncSession<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    type Target = AsyncConnection<AS, ASTU>;

    fn deref(&self) -> &AsyncConnection<AS, ASTU> {
        &self.conn
    }
}

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L110-L114
impl<AS, ASTU> DerefMut for AsyncSession<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    fn deref_mut(&mut self) -> &mut AsyncConnection<AS, ASTU> {
        &mut self.conn
    }
}

impl<AS, ASTU> AsyncSession<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + UpgraderExtRefer<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    pub(crate) fn new(conn: AsyncConnection<AS, ASTU>) -> Self {
        // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L435-L439
        let (tx, rx) = mpsc::channel();
        Self {
            conn,
            unsolicited_responses: rx,
            unsolicited_responses_tx: tx,
        }
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L461-L468
    pub async fn select<S: AsRef<str>>(&mut self, mailbox_name: S) -> Result<Mailbox> {
        self.run_command_and_read_response(&format!(
            "SELECT {}",
            validate_str(mailbox_name.as_ref())?
        ))
        .await
        .and_then(|lines| parse_mailbox(&lines[..], &mut self.unsolicited_responses_tx))
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L540-L551
    pub async fn fetch<S1, S2>(&mut self, sequence_set: S1, query: S2) -> ZeroCopyResult<Vec<Fetch>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        self.run_command_and_read_response(&format!(
            "FETCH {} {}",
            sequence_set.as_ref(),
            query.as_ref()
        ))
        .await
        .and_then(|lines| parse_fetches(lines, &mut self.unsolicited_responses_tx))
    }
}
