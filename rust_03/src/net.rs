use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::chat::{self, Role};
use crate::dh::{self, KeyPair};

pub fn run_server(port: u16) -> Result<(), Box<dyn Error>> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("[SERVER] Listening on {addr}");
    println!("[SERVER] Waiting for client...");

    // ====== IMPORTANT : afficher les paramètres DH dès le démarrage ======
    println!("[DH] Using hardcoded DH parameters:");
    let p = dh::P;
    println!(
        "  p = {:04X} {:04X} {:04X} {:04X} (64-bit prime - public)",
        (p >> 48) as u16,
        ((p >> 32) & 0xFFFF) as u16,
        ((p >> 16) & 0xFFFF) as u16,
        (p & 0xFFFF) as u16
    );
    println!("  g = {} (generator - public)", dh::G);
    // =====================================================================

    let (mut stream, client_addr) = listener.accept()?;
    println!("[SERVER] Client connected from {client_addr}");

    let secret = dh_server_handshake(&mut stream)?;

    println!("[SERVER] Shared secret established, starting chat...");
    chat::run_chat(stream, secret, Role::Server)
}

pub fn run_client(addr: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr)?;
    println!("[CLIENT] Connected to {addr}");

    let secret = dh_client_handshake(&mut stream)?;

    println!("[CLIENT] Shared secret established, starting chat...");
    chat::run_chat(stream, secret, Role::Client)
}

fn dh_server_handshake(stream: &mut TcpStream) -> Result<u64, Box<dyn Error>> {
    println!("[DH][SERVER] Starting key exchange...");

    let keypair = KeyPair::generate();

    let public_bytes = keypair.public.to_be_bytes();
    println!("[DH][SERVER] Sending public key: {:016X}", keypair.public);
    send_all(stream, &public_bytes)?;

    let other_bytes = recv_exact(stream, 8)?;
    let client_public = u64::from_be_bytes(other_bytes.try_into().unwrap());
    println!(
        "[DH][SERVER] Received client public key: {:016X}",
        client_public
    );

    let secret = dh::compute_shared_secret(keypair.private, client_public);
    println!("[DH][SERVER] Shared secret = {secret:016X}");
    Ok(secret)
}

fn dh_client_handshake(stream: &mut TcpStream) -> Result<u64, Box<dyn Error>> {
    println!("[DH][CLIENT] Starting key exchange...");

    let keypair = KeyPair::generate();

    let other_bytes = recv_exact(stream, 8)?;
    let server_public = u64::from_be_bytes(other_bytes.try_into().unwrap());
    println!(
        "[DH][CLIENT] Received server public key: {:016X}",
        server_public
    );

    let public_bytes = keypair.public.to_be_bytes();
    println!("[DH][CLIENT] Sending public key: {:016X}", keypair.public);
    send_all(stream, &public_bytes)?;

    let secret = dh::compute_shared_secret(keypair.private, server_public);
    println!("[DH][CLIENT] Shared secret = {secret:016X}");
    Ok(secret)
}

pub fn send_all(stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn Error>> {
    stream.write_all(data)?;
    Ok(())
}

pub fn recv_exact(stream: &mut TcpStream, len: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = vec![0u8; len];
    let mut read_total = 0;

    while read_total < len {
        let n = stream.read(&mut buf[read_total..])?;
        if n == 0 {
            return Err("Connection closed before receiving enough data".into());
        }
        read_total += n;
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_dh_handshake_over_tcp() {
        // On choisit une adresse locale avec port 0 (le système choisit un port libre)
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let addr = listener.local_addr().unwrap();

        // On a besoin de déplacer le listener dans le thread serveur
        let server_handle = thread::spawn(move || {
            // accepter une seule connexion
            let (mut stream, _) = listener.accept().expect("accept failed");
            dh_server_handshake(&mut stream).expect("server handshake failed")
        });

        // Côté client, on se connecte
        let client_handle = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("connect failed");
            dh_client_handshake(&mut stream).expect("client handshake failed")
        });

        let secret_server = server_handle.join().expect("server thread panicked");
        let secret_client = client_handle.join().expect("client thread panicked");

        assert_eq!(
            secret_server, secret_client,
            "Server and client must derive the same DH secret"
        );
        assert_ne!(secret_server, 0);
    }

    #[test]
    fn test_send_all_and_recv_exact() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let addr = listener.local_addr().unwrap();

        let server_handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept failed");
            let data = recv_exact(&mut stream, 5).expect("recv_exact failed");
            data
        });

        let client_handle = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("connect failed");
            let bytes = b"hello";
            send_all(&mut stream, bytes).expect("send_all failed");
        });

        let received = server_handle.join().expect("server thread panicked");
        client_handle.join().expect("client thread panicked");

        assert_eq!(received, b"hello");
    }
}

