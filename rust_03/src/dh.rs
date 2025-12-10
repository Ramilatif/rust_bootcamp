use rand::Rng;

/// Hardcoded 64-bit prime (public)
pub const P: u64 = 0xD87F_A3E2_91B4_C7F3;
/// Generator (public)
pub const G: u64 = 2;

#[derive(Debug, Clone, Copy)]
pub struct KeyPair {
    pub private: u64,
    pub public: u64,
}

impl KeyPair {
    /// Génère une paire (private, public)
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        // Clé privée aléatoire 64 bits
        // Attention: `gen` est un mot-clé dans ta version → raw ident `r#gen`
        let private: u64 = rng.r#gen();

        // public = g^private mod p
        let public = modular_pow(G as u128, private as u128, P as u128) as u64;

        println!("[DH] Generated keypair:");
        println!("  private = {private:016X}");
        println!("  public  = {public:016X}");

        Self { private, public }
    }
}

/// Calcule le secret partagé à partir de notre private et de leur public
pub fn compute_shared_secret(our_private: u64, their_public: u64) -> u64 {
    // secret = (their_public ^ our_private) mod P
    let secret =
        modular_pow(their_public as u128, our_private as u128, P as u128) as u64;

    println!("[DH] Computed shared secret = {secret:016X}");
    secret
}

/// Exponentiation modulaire (square and multiply)
pub fn modular_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 {
        return 0;
    }

    let mut result: u128 = 1;
    base %= modulus;

    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exp >>= 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modular_pow_basic() {
        // 2^10 = 1024
        assert_eq!(modular_pow(2, 10, 1_000_000), 1024);
        // (2^10) mod 1000 = 24
        assert_eq!(modular_pow(2, 10, 1000), 24);
    }

    #[test]
    fn test_dh_shared_secret_matches() {
        let server = KeyPair::generate();
        let client = KeyPair::generate();

        let secret_server = compute_shared_secret(server.private, client.public);
        let secret_client = compute_shared_secret(client.private, server.public);

        assert_eq!(
            secret_server, secret_client,
            "Les deux côtés doivent obtenir le même secret"
        );
    }
}

