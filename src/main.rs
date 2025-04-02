use anyhow::bail;
use clap::Parser;
use iroh::{
    Endpoint, SecretKey,
    endpoint::{RecvStream, SendStream, VarInt},
};
use std::{io::ErrorKind, process::exit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
};

mod args;

const KEY_PATH: &str = "key.priv";
const ALPN: &[u8] = b"do-ssh";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = args::Cli::parse();

    match cli.sub {
        args::Sub::Client { node_id, port } => {
            let ep = Endpoint::builder()
                //.secret_key(key)
                .alpns(vec![ALPN.to_vec()])
                .discovery_n0()
                .discovery_local_network()
                .bind()
                .await?;
            let conn = ep.connect(node_id, ALPN).await?;
            let (mut tx, mut rx) = conn.open_bi().await?;
            tx.write_u16(port).await?;
            /*tokio::spawn(async move {
                if let Err(err) = tokio::io::copy(&mut rx, &mut tokio::io::stdout()).await {
                    eprintln!("{}", err);
                }
            });*/
            let (ctx, mut crx) = tokio::sync::mpsc::channel(1);
            let ctx_ = ctx.clone();
            tokio::spawn(async move {
                if let Err(err) = tokio::io::copy(&mut rx, &mut tokio::io::stdout()).await {
                    _ = ctx_.send(err.to_string()).await;
                }
            });
            let ctx_ = ctx.clone();
            tokio::spawn(async move {
                if let Err(err) = tokio::io::copy(&mut tokio::io::stdin(), &mut tx).await {
                    _ = ctx_.send(err.to_string()).await;
                }
            });
            tokio::spawn(async move {
                let closed = conn.closed().await.to_string();
                eprintln!("Got closing");
                _ = ctx.send(closed).await;
            });
            let error = crx.recv().await.unwrap_or("wtf".to_string());
            eprintln!("{}", error);
            exit(1);
        }
        args::Sub::Server { key_file, create } => {
            let key = match tokio::fs::read(KEY_PATH).await {
                Ok(file) => {
                    //  Parse key
                    let bytes: [u8; 32] = (*file.into_boxed_slice()).try_into()?;
                    Ok(SecretKey::from_bytes(&bytes))
                }
                Err(err) if err.kind() == ErrorKind::NotFound && create => {
                    //  Generate new key
                    println!("Generating new key at {}", key_file.to_str().unwrap_or(""));
                    let bytes: [u8; 32] = rand::random();
                    tokio::fs::write(&key_file, &bytes).await?;
                    Ok(SecretKey::from_bytes(&bytes))
                }
                //  Fuck
                Err(err) => Err(anyhow::Error::new(err)),
            }?;
            let ep = Endpoint::builder()
                .secret_key(key)
                .alpns(vec![ALPN.to_vec()])
                .discovery_n0()
                .discovery_local_network();
            let ep = ep.bind().await?;
            println!("NodeId: {}", ep.node_id());
            loop {
                //  Can be unwrapped, because we won't be here if the endpoint gets closed
                //    (This only returns None if the endpoint is closed)
                let connection = ep.accept().await.unwrap();
                tokio::spawn(async move {
                    let connection = match connection.await {
                        Ok(val) => val,
                        Err(err) => {
                            eprintln!("{}", err);
                            return;
                        }
                    };
                    let (tx, rx) = match connection.accept_bi().await {
                        Ok(val) => val,
                        Err(err) => {
                            eprintln!("{}", err);
                            return;
                        }
                    };
                    println!("Got connection: {}", connection.remote_node_id().unwrap());
                    if let Err(err) = handle_connection(tx, rx).await {
                        connection.close(VarInt::from(1u8), &format!("{}", err).into_bytes());
                        _ = connection.closed().await;

                        eprintln!("{}", err);
                    } else {
                        connection.close(VarInt::from(0u8), &[]);
                    }
                    println!(
                        "Closed connection: {}",
                        connection.remote_node_id().unwrap()
                    );
                });
            }
        }
        args::Sub::Gen {
            key_file,
            r#override,
        } => {
            if key_file.is_file() && !r#override {
                bail!("The keyfile already exists!");
            }
            println!(
                "Generating new key at {}",
                key_file
                    .file_name()
                    .and_then(|val| val.to_str())
                    .unwrap_or("None")
            );
            let bytes: [u8; 32] = rand::random();
            tokio::fs::write(&key_file, &bytes).await?;
        }
    };
    Ok(())
}

async fn handle_connection(mut tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    let port = rx.read_u16().await?;
    let mut tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    println!("Created new connection with port {}", port);
    let (mut tcprx, mut tcptx) = tcp.split();

    let (res1, res2) = join!(
        tokio::io::copy(&mut tcprx, &mut tx),
        tokio::io::copy(&mut rx, &mut tcptx)
    );
    res1?;
    res2?;
    Ok(())
}
