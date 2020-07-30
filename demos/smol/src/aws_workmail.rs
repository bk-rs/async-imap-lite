/*
cargo run -p async-imap-lite-demo-smol --bin aws_workmail us-west-2 foo@example.com '123456'
*/

// https://docs.aws.amazon.com/workmail/latest/userguide/using_IMAP_client.html

use std::env;
use std::io;

use futures_lite::future::block_on;

use async_imap_lite::{connect_async, AsyncTlsClientTlsUpgrader};

fn main() -> io::Result<()> {
    block_on(run())
}

async fn run() -> io::Result<()> {
    let region = env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("REGION").unwrap_or_else(|_| "us-west-2".to_owned()));
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| env::var("USERNAME").unwrap_or_else(|_| "foo@example.com".to_owned()));
    let password = env::args()
        .nth(3)
        .unwrap_or_else(|| env::var("PASSWORD").unwrap_or_else(|_| "123456".to_owned()));

    //
    let port: u16 = 993;
    let endpoint = format!("imap.mail.{}.awsapps.com", region);

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
