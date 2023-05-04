use self::crypto::Hash;
use crate::client::Client;
use crate::peer::Peer;

use std::io;
// use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::{
    // collections::HashMap,
    sync::mpsc::{self},
    thread,
};
// use storage::Storage;
use ursa::signatures::{prelude::Ed25519Sha512, SignatureScheme};
mod client;
mod comands;
mod crypto;
mod peer;
mod storage;

pub fn run_cli(client: Arc<Mutex<Client>>) {
    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();
        let command_parts: Vec<&str> = buffer.split_whitespace().collect();

        if !command_parts.is_empty() {
            let command = match command_parts.len() {
                1 => command_parts[0].to_string(),
                _ => format!("{} {}", command_parts[0], command_parts[1]),
            };

            match command.as_str() {
                "create account" => match client.lock().unwrap().create_account() {
                    Ok(()) => println!("Account created."),
                    Err(err) => println!("Error: {:?}", err),
                },
                "add funds" => {
                    if command_parts.len() < 4 {
                        println!("Usage: add funds <account_id> <value> <asset_id>");
                    } else {
                        let account_id = command_parts[2].parse().unwrap();
                        let value = command_parts[3].parse().unwrap();
                        let asset_id = command_parts[4].to_string();

                        match client
                            .lock()
                            .unwrap()
                            .add_funds(account_id, value, asset_id)
                        {
                            Ok(()) => println!("Funds added."),
                            Err(err) => println!("Error: {:?}", err),
                        }
                    }
                }
                "exit" => {
                    println!("Exiting...");
                    break;
                }
                "help" => {
                    println!("Available commands:");
                    println!("create account: Create a new account.");
                    println!("add funds <account_id> <value> <asset_id>: Add funds to an account.");
                    println!("exit: Exit the program.");
                }
                _ => println!("Unknown command. Type 'help' for a list of commands."),
            }
        } else {
            println!("No command entered. Type 'help' for a list of commands.");
        }
    }
}

fn main() {
    let (client_to_peer_tx, peer_rx) = mpsc::channel();
    let (peer_to_client_tx, client_rx) = mpsc::channel();

    let (public_key, private_key) = Ed25519Sha512::new().keypair(None).unwrap();

    let client_to_peer_tx_clone = client_to_peer_tx.clone();
    let client = Arc::new(Mutex::new(Client::new(
        client_to_peer_tx_clone,
        public_key.clone(),
        private_key,
        client_rx,
    )));

    let peer = Peer::new(
        1,
        peer_rx,
        vec![client_to_peer_tx],
        50,
        vec![public_key],
        peer_to_client_tx,
    );

    let peer_handle = thread::spawn(move || {
        peer.start().unwrap();
    });

    let client_clone = Arc::clone(&client);
    let client_handle = thread::spawn(move || {
        client_clone.lock().unwrap().receive_updates();
    });

    run_cli(client);

    peer_handle.join().unwrap();
    client_handle.join().unwrap();
}
