use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
struct Args {
    /// bind the service to this tcp port, default 5555
    #[arg(short, long, default_value = "5555")]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let s = format!("0.0.0.0:{}", args.port)
        .parse::<SocketAddr>()
        .unwrap();
    println!("Listening to {s}");
    let listener = TcpListener::bind(s).unwrap();
    for incoming in listener.incoming() {
        match incoming {
            Ok(incoming) => {
                thread::spawn(|| proxy_hack(incoming));
            }

            Err(e) => eprintln!("error {e}"),
        }
    }
}

fn proxy_hack(mut downstream: TcpStream) {
    let peer_addr = downstream.peer_addr().unwrap();

    println!("{peer_addr} - connected, connecting to upstream");

    let upstream = TcpStream::connect("chat.protohackers.com:16963").unwrap();

    let parsing_regex = Regex::new("7[1-9a-zA-Z]{25,35}").unwrap();

    // forward anytinng sent by downstream to upstream
    thread::spawn({
        let mut upstream = upstream.try_clone().unwrap();
        let mut downstream = downstream.try_clone().unwrap();
        let peer_addr = peer_addr.clone();
        move || {
            println!("{peer_addr} - forwarding to upstream started!");
            let mut buf = [0u8; 4096];
            while let Ok(count) = downstream.read(&mut buf) {
                if count == 0 {
                    //EOF
                    break;
                }
                if let Err(_) = upstream.write_all(&buf[0..count]) {
                    // on error occured writting to upstream, close connection
                    break;
                }
            }
            println!("{peer_addr} - forwarding to upstream ended!");
        }
    });

    // forward anything from upstream to downstream ; hacking BogusCoin addr
    // to help parsing, act line by line

    println!("{peer_addr} - forwarding to downstream started!");
    let upstream = BufReader::new(upstream);
    for line in upstream.lines() {
        if let Ok(line) = line {
            let line = parsing_regex
                .replace_all(&line, "7YWHMfk9JZe0LM0g1ZauHuiSxhI")
                .into_owned();
            if let Err(_) = downstream.write_all(line.as_bytes()) {
                break;
            }
        } else {
            break;
        }
    }
    println!("{peer_addr} - forwarding to downstream ended!");
}
