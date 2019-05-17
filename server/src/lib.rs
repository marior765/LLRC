extern crate tokio;

use tokio::io::{copy, write_all, read_exact};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::thread;
use std::net::{
    TcpStream as StdTcpStream, 
    TcpListener as StdTcpListener
};
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;
use futures::Async;

const MSG_SIZE: usize = 30;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

pub fn init_server(addr: &str) {
    let server = StdTcpListener::bind(addr).expect("Stream failed to connect");
    server.set_nonblocking(true).expect("failed to initiate non-blocking");

    let handle = tokio::reactor::Handle::default();

    let mut clients = vec![];
    let (transmitter, receiver) = mpsc::channel::<String>();
    let mut server = TcpListener::from_std(server, &handle).unwrap(); 
    // .and_then(|s| {
        'server: loop {
            if let Ok(Async::Ready((socket, addr))) = server.poll_accept() {
                println!("Client {} connected", addr);

                let transmitter = transmitter.clone();
                clients.push(socket.try_clone().expect("failed to clone"));

                thread::spawn(move || loop {
                    let buff = vec![0; MSG_SIZE];

                    // read_exact(socket, &mut buff);
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("failed to convert utf8");

                    println!("{}: {:?}", addr, msg);
                    transmitter.send(msg).expect("failed to send msg to receiver");

                    sleep();
                });
            }

            if let Ok(msg) = receiver.try_recv() {
                clients = clients.into_iter().filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                }).collect::<Vec<_>>();
            }

            sleep();
        }
}

pub fn init_client(addr: &str) {
    // let addr = addr.parse().unwrap();
    let mut client = StdTcpStream::connect(addr).expect("Stream failed to connect");
    &client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let handle = tokio::reactor::Handle::default();

    let (transmitter, receiver) = mpsc::channel::<String>();

    TcpStream::from_std(client.try_clone().unwrap(), &handle).and_then(|stream| {
        // thread::spawn(move || loop {
            let mut buff = vec![0; MSG_SIZE];
            read_exact(stream, &mut buff);
            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
            println!("message received: {:?}", {});

            match receiver.try_recv() {
                Ok(msg) => {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).expect("writing to socket failed");
                    println!("message sent {:?}", msg);
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => ()
            }

            thread::sleep(Duration::from_millis(100));
        // });

        println!("write a Message:");
        loop {
            let mut buff = String::new();
            std::io::stdin().read_line(&mut buff).expect("reading from stdin failed");
            let msg = buff.trim().to_string();
            if msg == ":quit" || transmitter.send(msg).is_err() { break }
        }
        Ok(())
    });
}

pub fn init_connection(addr: &'static str) {
    init_server(addr); 
    init_client(addr);
}