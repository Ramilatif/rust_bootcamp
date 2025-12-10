use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

use crate::cipher::{self, xor_with_keystream};
use crate::net::{recv_exact, send_all};

#[derive(Debug, Clone, Copy)]
pub enum Role {
    Server,
    Client,
}

pub fn run_chat(stream: TcpStream, shared_secret: u64, role: Role) -> Result<(), Box<dyn Error>> {
    println!("[STREAM] Generating keystream from secret {shared_secret:016X}...");
    let (mut ks_send, mut ks_recv) = cipher::make_streams(shared_secret, role);

    let mut send_stream = stream.try_clone()?;
    let mut recv_stream = stream;

    // Thread RÉCEPTION (renvoie `()`, pas de Result -> plus de problème de Send)
    let recv_handle = thread::spawn(move || {
        loop {
            let len_bytes = match recv_exact(&mut recv_stream, 4) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[CHAT][RECV] Error or connection closed: {e}");
                    break;
                }
            };
            let len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
            if len == 0 {
                continue;
            }

            let cipher = match recv_exact(&mut recv_stream, len) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[CHAT][RECV] Error reading payload: {e}");
                    break;
                }
            };

            let plain = xor_with_keystream(&cipher, &mut ks_recv);
            let msg = String::from_utf8_lossy(&plain);
            println!("\n[PEER] {msg}");
            print!("> ");
            let _ = std::io::stdout().flush();
        }
    });

    // Thread ENVOI
    let send_handle = thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin);

        println!("[CHAT] Secure channel established! Type messages (or /quit):");
        print!("> ");
        let _ = std::io::stdout().flush();

        loop {
            let mut line = String::new();
            let n = match reader.read_line(&mut line) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[CHAT][SEND] Error reading stdin: {e}");
                    break;
                }
            };
            if n == 0 {
                break;
            }

            let line_trimmed = line.trim_end().to_string();

            if line_trimmed.is_empty() {
                print!("> ");
                let _ = std::io::stdout().flush();
                continue;
            }

            if line_trimmed == "/quit" {
                println!("[CHAT] Closing connection...");
                break;
            }

            let plain_bytes = line_trimmed.as_bytes();
            let cipher = xor_with_keystream(plain_bytes, &mut ks_send);

            let len = cipher.len() as u32;
            let len_bytes = len.to_be_bytes();

            if let Err(e) = send_all(&mut send_stream, &len_bytes) {
                eprintln!("[CHAT][SEND] Error sending length: {e}");
                break;
            }
            if let Err(e) = send_all(&mut send_stream, &cipher) {
                eprintln!("[CHAT][SEND] Error sending data: {e}");
                break;
            }

            print!("> ");
            let _ = std::io::stdout().flush();
        }
    });

    let _ = send_handle.join();
    let _ = recv_handle.join();

    println!("[CHAT] Connection closed.");
    Ok(())
}

