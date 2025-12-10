use crate::chat::Role;

/// Linear Congruential Generator (LCG) basé sur un seed 32 bits
#[derive(Debug, Clone)]
pub struct Lcg {
    state: u32,
}

impl Lcg {
    /// Initialise le LCG avec un seed (par ex: secret partagé DH)
    pub fn new(seed: u64) -> Self {
        // On ne garde que 32 bits pour l'état interne
        Self {
            state: (seed & 0xFFFF_FFFF) as u32,
        }
    }

    /// Génère un octet de keystream
    pub fn next_byte(&mut self) -> u8 {
        const A: u64 = 1103515245;
        const C: u64 = 12345;
        const M: u64 = 1u64 << 32;

        let next = (A * self.state as u64 + C) % M;
        self.state = next as u32;

        (self.state & 0xFF) as u8
    }
}

/// XOR un buffer avec le keystream
pub fn xor_with_keystream(data: &[u8], ks: &mut Lcg) -> Vec<u8> {
    data.iter()
        .map(|b| b ^ ks.next_byte())
        .collect()
}

/// Crée deux flux keystream (envoi / réception) à partir du secret DH.
pub fn make_streams(secret: u64, role: Role) -> (Lcg, Lcg) {
    const S1: u64 = 0xAAAA_AAAA_AAAA_AAAA;
    const S2: u64 = 0x5555_5555_5555_5555;

    match role {
        Role::Server => (Lcg::new(secret ^ S1), Lcg::new(secret ^ S2)),
        Role::Client => (Lcg::new(secret ^ S2), Lcg::new(secret ^ S1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_round_trip() {
        let secret = 0x1234_5678_9ABC_DEF0u64;

        let (mut ks_send_server, mut ks_recv_server) = make_streams(secret, Role::Server);
        let (mut ks_send_client, mut ks_recv_client) = make_streams(secret, Role::Client);

        let msg = b"Hello Rust!";

        // Serveur envoie
        let cipher = xor_with_keystream(msg, &mut ks_send_server);
        // Client reçoit et déchiffre
        let plain = xor_with_keystream(&cipher, &mut ks_recv_client);
        assert_eq!(plain, msg);

        // Client répond
        let reply = b"Hi!";
        let cipher2 = xor_with_keystream(reply, &mut ks_send_client);
        // Serveur déchiffre
        let plain2 = xor_with_keystream(&cipher2, &mut ks_recv_server);
        assert_eq!(plain2, reply);
    }
}

