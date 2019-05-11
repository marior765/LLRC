extern crate tokio;

use tokio::io::{copy, write_all};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::thread;

pub fn init_server(addr: &str) {
    let addr = addr.parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener.incoming()
        .map_err(|e| println!("Error: {}", e))
        .for_each(|s| {
            let (reader, writer) = s.split();

            let copied_bytes = copy(reader, writer);

            let msg = copied_bytes.then(move |result| {
                match result {
                    Ok((amt, _, _)) => println!("wrote {} bytes", amt),
                    Err(e) => println!("error: {}", e),
                }

                Ok(())
            });

            tokio::spawn(msg)
        });

        tokio::run(server);
}

pub fn init_client(addr: &str) {
    let addr = addr.parse().unwrap();
    let client = TcpStream::connect(&addr).and_then(|stream| {
        println!("created stream");

        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();
 
        write_all(stream, input).then(|result| {
            println!("wrote to stream; success={:?}", result.is_ok());
            Ok(())
        })

    })
    .map_err(|err| {
        println!("connection error = {:?}", err);
    });

    println!("About to create the stream and write to it...");
    tokio::run(client);
    println!("Stream has been created and written to.");
}

pub fn init_connection(addr: &'static str) {
    init_server(addr); 
    init_client(addr);
}