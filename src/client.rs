use std::ops::{Deref, DerefMut};
use std::result;

use crate::connection::{
    AsRawFdOrSocket, Async, AsyncConnection, AsyncRead, AsyncWrite, TlsClientUpgrader,
    UpgraderExtRefer,
};
use crate::imap::{validate_str, Error};
use crate::session::AsyncSession;

pub struct AsyncClient<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    conn: AsyncConnection<AS, ASTU>,
}

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L88-L94
impl<AS, ASTU> Deref for AsyncClient<AS, ASTU>
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

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L96-L100
impl<AS, ASTU> DerefMut for AsyncClient<AS, ASTU>
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

// ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L226-L233
macro_rules! ok_or_unauth_client_err {
    ($r:expr, $self:expr) => {
        match $r {
            Ok(o) => o,
            Err(e) => return Err((e, $self)),
        }
    };
}

impl<AS, ASTU> AsyncClient<AS, ASTU>
where
    AS: AsRawFdOrSocket,
    Async<AS>: AsyncRead + AsyncWrite,
    ASTU: TlsClientUpgrader<Async<AS>> + UpgraderExtRefer<Async<AS>> + Unpin,
    ASTU::Output: AsyncRead + AsyncWrite + Unpin,
{
    pub(crate) fn new(conn: AsyncConnection<AS, ASTU>) -> Self {
        Self { conn }
    }

    // ref https://github.com/jonhoo/rust-imap/blob/v2.2.0/src/client.rs#L307-L320
    pub async fn login<U: AsRef<str>, P: AsRef<str>>(
        mut self,
        username: U,
        password: P,
    ) -> result::Result<AsyncSession<AS, ASTU>, (Error, Self)> {
        let u = ok_or_unauth_client_err!(validate_str(username.as_ref()), self);
        let p = ok_or_unauth_client_err!(validate_str(password.as_ref()), self);
        ok_or_unauth_client_err!(
            self.run_command_and_check_ok(&format!("LOGIN {} {}", u, p))
                .await,
            self
        );

        Ok(AsyncSession::new(self.conn))
    }
}
