use clap::Parser;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{borrow::Cow, error::Error, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Parser)]
struct Args {
    /// bind the service to this tcp port, default 5555
    #[arg(short, long, default_value = "5555")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let s = format!("0.0.0.0:{}", args.port)
        .parse::<SocketAddr>()
        .unwrap();
    println!("Listening to {s}");
    let listener = TcpListener::bind(s).await?;
    while let Ok((downstream, remote_addr)) = listener.accept().await {
        tokio::spawn(handle_downstream(downstream, remote_addr));
    }
    Ok(())
}

async fn handle_downstream(downstream: TcpStream, remote: SocketAddr) {
    match do_handle_downstream(downstream, remote).await {
        Ok(_) => println!("{remote} connection closed without error"),
        Err(e) => println!("{remote} connection closed with error: {e}"),
    }
}

async fn do_handle_downstream(
    mut downstream: TcpStream,
    remote: SocketAddr,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    format!("{remote} - incoming connection");
    // connect to upstream
    let mut upstream = TcpStream::connect("chat.protohackers.com:16963").await?;
    format!("{remote} - connected to upstream");

    let (down_reader, mut down_writer) = downstream.split();
    let (up_reader, mut up_writer) = upstream.split();

    let up_reader = BufReader::new(up_reader);
    let down_reader = BufReader::new(down_reader);

    let mut up_lines = up_reader.lines();
    let mut down_lines = down_reader.lines();

    loop {
        tokio::select! {
            up_line = up_lines.next_line() => {
                let line = up_line?;
                match line {
                    Some(line) => {
                        println!("{remote} - upstream received {line}");
                        let line = replace(&line);
                        down_writer.write_all(&line.as_bytes()).await?;
                        down_writer.write_all(b"\n").await?;
                },
                    None => break
                }
            }
            down_line = down_lines.next_line() => {
                let line = down_line?;
                match line {
                    Some(line) => {
                        println!("{remote} - downstream received {line}");
                        let line = replace(&line);
                        up_writer.write_all(&line.as_bytes()).await?;
                        up_writer.write_all(b"\n").await?;
                    },
                    None => break
                }
            }
        }
    }

    Ok(())
}

lazy_static! {
    static ref REGEX: Regex = Regex::new("^7[0-9a-zA-Z]{25,34}$").unwrap();
}

fn replace(text: &str) -> String {
    text.split(' ')
        .map(|txt| REGEX.replace(txt, "7YWHMfk9JZe0LM0g1ZauHuiSxhI"))
        .join(" ")
}

#[test]
fn test() {
    assert_eq!(replace("REGEX"), "REGEX");
    assert_eq!(
        replace("7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"),
        "7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"
    );

    assert_eq!(
        replace("7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"),
        "7YWHMfk9JZe0LM0g1ZauHuiSxhI"
    );
    assert_eq!(
        replace(" 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"),
        " 7YWHMfk9JZe0LM0g1ZauHuiSxhI"
    );
    assert_eq!(
        replace(" 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T "),
        " 7YWHMfk9JZe0LM0g1ZauHuiSxhI "
    );
    assert_eq!(
        replace("foo 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"),
        "foo 7YWHMfk9JZe0LM0g1ZauHuiSxhI"
    );
    assert_eq!(
        replace("7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T bar"),
        "7YWHMfk9JZe0LM0g1ZauHuiSxhI bar"
    );
    assert_eq!(
        replace("7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"),
        "7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI"
    );
    assert_eq!(replace("7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T 7adNeSwJkMakpEcln9HEtthSRtxdmEHOT8T"), 
    "7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI");
}
