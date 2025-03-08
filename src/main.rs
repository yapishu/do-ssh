use anyhow::bail;
use iroh::{Endpoint, NodeId, SecretKey};
use std::{io::Read, str::FromStr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
};

const KEY_PATH: &str = "key.priv";
const ALPN: &[u8] = b"do-ssh";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = std::env::args()
        .nth(1)
        .and_then(|val| NodeId::from_str(&val).ok());
    let port = std::env::args()
        .nth(2)
        .and_then(|val| u16::from_str(&val).ok())
        .unwrap_or(22);
    if let Some(node) = node {
        let ep = Endpoint::builder()
            //.secret_key(key)
            .alpns(vec![ALPN.to_vec()])
            .discovery_n0()
            .discovery_local_network()
            .bind()
            .await?;
        let conn = ep.connect(node, ALPN).await?;
        let (mut tx, mut rx) = conn.open_bi().await?;
        tx.write_u8(0).await?;
        tokio::spawn(async move {
            if let Err(err) = tokio::io::copy(&mut rx, &mut tokio::io::stdout()).await {
                eprintln!("{}", err);
            }
        });
        tokio::io::copy(&mut tokio::io::stdin(), &mut tx).await?;

        Ok(())
    } else {
        let key = read_key().await.unwrap_or({
            println!("Generating new key...");
            let key = SecretKey::generate(rand::rngs::OsRng);
            tokio::fs::write("key.priv", key.clone().to_bytes()).await?;
            key
        });
        let ep = Endpoint::builder()
            .secret_key(key)
            .alpns(vec![ALPN.to_vec()])
            .discovery_n0()
            .discovery_local_network();
        let ep = ep.bind().await?;
        println!("Node: {}", ep.node_id());
        loop {
            let conn = ep.accept().await.unwrap();
            tokio::spawn(async move {
                let conn = conn.await?;
                let (mut tx, mut rx) = conn.accept_bi().await?;
                if rx.read_u8().await? != 0 {
                    panic!();
                }
                let mut tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
                let (mut tcprx, mut tcptx) = tcp.split();
                let (res1, res2) = join!(
                    tokio::io::copy(&mut tcprx, &mut tx),
                    tokio::io::copy(&mut rx, &mut tcptx)
                );
                res1?;
                res2?;

                Ok::<(), anyhow::Error>(())
            });
        }
    }
}

async fn read_key() -> anyhow::Result<SecretKey> {
    let key = tokio::fs::read(KEY_PATH).await?;
    if key.len() != 32 {
        bail!("Invalid key length");
    }
    let bytes: [u8; 32] = (*key.into_boxed_slice()).try_into().unwrap();
    Ok(SecretKey::from_bytes(&bytes))
}
