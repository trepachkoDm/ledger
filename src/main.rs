use self::crypto::Hash;
use crate::client::Client;
use crate::peer::Peer;
use std::{collections::HashMap, sync::mpsc::channel, thread};
use ursa::signatures::{prelude::Ed25519Sha512, SignatureScheme};
mod client;
mod comands;
mod crypto;
mod peer;
mod storage;

fn main() {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let (tx3, rx3) = channel();
    let (tx2_1, tx3_1) = (tx2.clone(), tx3.clone());
    let (tx1_2, tx3_2) = (tx1.clone(), tx3.clone());
    let (tx1_3, tx2_3) = (tx1.clone(), tx2.clone());
    let (public_key1, private_key1) = Ed25519Sha512::new().keypair(None).unwrap();
    let (public_key2, private_key2) = Ed25519Sha512::new().keypair(None).unwrap();
    let (public_key3, private_key3) = Ed25519Sha512::new().keypair(None).unwrap();

    let _public_key1_clone = public_key1.clone();
    let public_key2_clone = public_key2.clone();
    let public_key3_clone = public_key3.clone();

    let peer1_thread_handle = thread::spawn(move || {
        let peer1 = Peer::new(
            1,
            rx1,
            vec![tx2_1, tx3_1],
            50,
            vec![public_key2_clone, public_key3_clone],
        );
        peer1.start().unwrap();
    });

    let public_key1_clone = public_key1.clone();
    let _public_key2_clone = public_key2.clone();
    let public_key3_clone = public_key3.clone();

    let peer2_thread_handle = thread::spawn(move || {
        let peer2 = Peer::new(
            2,
            rx2,
            vec![tx1_2, tx3_2],
            40,
            vec![public_key1_clone, public_key3_clone],
        );
        peer2.start().unwrap();
    });

    let public_key1_clone = public_key1.clone();
    let public_key2_clone = public_key2.clone();
    let _public_key3_clone = public_key3.clone();

    let peer3_thread_handle = thread::spawn(move || {
        let peer3 = Peer::new(
            3,
            rx3,
            vec![tx1_3, tx2_3],
            25,
            vec![public_key1_clone, public_key2_clone],
        );
        peer3.start().unwrap();
    });

    let peer1_client = Client::new(tx1, public_key1, private_key1);
    let peer2_client = Client::new(tx2, public_key2, private_key2);
    let peer3_client = Client::new(tx3, public_key3, private_key3);

    peer1_client.create_account().unwrap();
    println!("{:?}", peer1_client);
    peer2_client.create_account().unwrap();
    println!("{:?}", peer2_client);
    peer3_client.create_account().unwrap();
    println!("{:?}", peer3_client);

    peer1_client
        .transfer_funds(1, 2, 13, String::from("Asset1"))
        .unwrap();
    println!("{:?}", peer1_client);
    peer2_client
        .update_account(2, Some(String::from("Saroza")), None)
        .unwrap();
    println!("{:?}", peer2_client);

    let mut params = HashMap::new();
    params.insert(String::from("param1"), String::from("value1"));
    params.insert(String::from("param2"), String::from("value2"));
    peer1_client
        .execute_smart_contract(1, String::from("contract_id"), params)
        .unwrap();
    println!("{:?}", peer1_client);

    peer3_client
        .release_asset(3, String::from("Asset2"), 26, String::from("AssetRel"))
        .unwrap();
    println!("{:?}", peer3_client);

    peer2_client
        .transfer_asset(2, 3, String::from("Asset1"))
        .unwrap();
    println!("{:?}", peer2_client);

    peer1_client
        .redeem_asset(1, String::from("Asset3"), 14, String::from("Asset4"))
        .unwrap();
    println!("{:?}", peer1_client);

    peer1_thread_handle.join().expect("peer1 failed");
    peer2_thread_handle.join().expect("peer2 failed");
    peer3_thread_handle.join().expect("peer3 failed");
}
