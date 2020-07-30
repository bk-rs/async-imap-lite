/*
cargo run -p async-imap-lite-demo-smol --bin gmail xxx@gmail.com '123456'
*/

// https://support.google.com/mail/answer/6386757
// https://support.google.com/mail/answer/7126229

// Enable IMAP
// https://mail.google.com/mail/u/0/#settings/fwdandpop

// Allow less secure apps: ON
// https://myaccount.google.com/u/0/lesssecureapps

// Allow
// https://accounts.google.com/b/0/DisplayUnlockCaptcha

use std::env;
use std::io;

use futures_lite::future::block_on;

use async_imap_lite::{connect_async, AsyncTlsClientTlsUpgrader};

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let username = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or_else(|_| "xxx@gmail.com".to_owned()));
    let password = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("PASSWORD").unwrap_or_else(|_| "123456".to_owned()));

    //
    let port: u16 = 993;
    let endpoint = "imap.gmail.com".to_owned();

    println!("endpoint: {}", endpoint);
    let addr = format!("{}:{}", endpoint, port);

    let tls_upgrader = AsyncTlsClientTlsUpgrader::new(Default::default(), endpoint);

    println!("connect");
    let client = connect_async(addr, tls_upgrader, Some(true))
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    println!("login");
    let mut session = client
        .login(username, password)
        .await
        .map_err(|(err, _)| io::Error::new(io::ErrorKind::Other, err))?;

    println!("select");
    session
        .select("Inbox")
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    println!("fetch");
    let messages = session
        .fetch("1", "RFC822")
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    println!("{:?}", messages.iter().next());

    println!("done");

    Ok(())
}
