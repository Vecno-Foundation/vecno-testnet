use crate::{block::Block, header::Header, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction};
use vecno_hashes::{Hash, ZERO_HASH};
use vecno_muhash::EMPTY_MUHASH;

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(0, Vec::new(), Vec::new(), 0, SUBNETWORK_ID_COINBASE, 0, self.coinbase_payload.to_vec())]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            Vec::new(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

/// The genesis block of the block-DAG which serves as the public transaction ledger for the main network.
pub const GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x00, 0x00, 0x25, 0xe2, 0x6c, 0xee, 0x95, 0x08, 0x22, 0x81, 0x90, 0xe4, 0x85, 0x66, 0xba, 0x31,
        0x96, 0x06, 0xe1, 0x8f, 0xb7, 0xe1, 0x74, 0x09, 0x1c, 0x89, 0x13, 0xbb, 0xee, 0xab, 0x7a, 0xcc
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x74, 0x50, 0x7e, 0xce, 0x7f, 0x65, 0xfa, 0xd6, 0x15, 0x3e, 0x58, 0x86, 0xc9, 0xad, 0x78, 0xa8,
        0x57, 0x7c, 0xd0, 0xdf, 0x09, 0x53, 0xb3, 0x22, 0x2d, 0x03, 0x8d, 0xd7, 0x73, 0xe6, 0x79, 0x7d
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 0x19409ce1deb,
    bits: 0x1e7fffff,
    nonce: 0x0000d885,
    daa_score: 0, // Checkpoint DAA score
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00, // Script version
        0x01, // Varint
        0x00, // OP-FALSE
        0x65, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x6c,
        0x79, 0x2c, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x65,
        0x76, 0x65, 0x72 // Eternally, for ever
    ],
};

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x55, 0xc2, 0xd4, 0x29, 0x9e, 0x21, 0xf9, 0x10, 0xd1, 0x57, 0x1d, 0x11, 0x49, 0x69, 0xce, 0xce, 0xf4, 0x8f, 0x9, 0xf9, 0x34,
        0xd4, 0x2c, 0xcb, 0x6a, 0x28, 0x1a, 0x15, 0x86, 0x8f, 0x29, 0x99,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x8e, 0xc8, 0x98, 0x56, 0x8c, 0x68, 0x1, 0xd1, 0x3d, 0xf4, 0xee, 0x6e, 0x2a, 0x1b, 0x54, 0xb7, 0xe6, 0x23, 0x6f, 0x67, 0x1f,
        0x20, 0x85, 0x4f, 0x5, 0x30, 0x64, 0x10, 0x51, 0x8e, 0xeb, 0x32,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 0x00,
    bits: 0x00,
    nonce: 0x00,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00, // Script version
        0x01,                                                                         // Varint
        0x00,                                                                         // OP-FALSE
        0x00, // vecno-testnet
    ],
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x41, 0x1f, 0x8c, 0xd2, 0x6f, 0x3d, 0x41, 0xae, 0xa3, 0x9e, 0x78, 0x57, 0x39, 0x27, 0xda, 0x24, 0xd2, 0x39, 0x95, 0x70, 0x5b,
        0x57, 0x9f, 0x30, 0x95, 0x9b, 0x91, 0x27, 0xe9, 0x6b, 0x79, 0xe3,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x19, 0x46, 0xd6, 0x29, 0xf7, 0xe9, 0x22, 0xa7, 0xbc, 0xed, 0x59, 0x19, 0x05, 0x21, 0xc3, 0x77, 0x1f, 0x73, 0xd3, 0x52, 0xdd,
        0xbb, 0xb6, 0x86, 0x56, 0x4a, 0xd7, 0xfd, 0x56, 0x85, 0x7c, 0x1b,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 0x17c5f62fbb6,
    bits: 0x207fffff,
    nonce: 0x2,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00, // Script version
        0x01,                                                                   // Varint
        0x00,                                                                   // OP-FALSE
        0x6b, 0x61, 0x73, 0x70, 0x61, 0x2d, 0x73, 0x69, 0x6d, 0x6e, 0x65, 0x74, // vecno-simnet
    ],
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        // Golang devnet genesis hash
        // 0xb3, 0x13, 0x87, 0x0a, 0x32, 0xc7, 0x04, 0xbd, 0xf1, 0x21, 0x4a, 0x3b, 0x27, 0x0c, 0xc4, 0x75, 0xd9, 0x42, 0xc2, 0x09, 0x2d,
        // 0x37, 0x9b, 0xc8, 0x70, 0x0a, 0xb0, 0x43, 0x31, 0x9e, 0xf8,
        // 0x46,
        // New rust devnet genesis hash updated according to the modified bits field (see below)
        0x4c, 0xb4, 0x8d, 0x0b, 0x20, 0x73, 0xb8, 0x02, 0x36, 0x01, 0x45, 0xa1, 0x5a, 0xd1, 0xab, 0xdc, 0x01, 0xd8, 0x9b, 0x5c, 0x2f,
        0xe4, 0x72, 0x26, 0x30, 0xab, 0x9b, 0x5f, 0xe9, 0xdf, 0xc4, 0xf2,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x58, 0xab, 0xf2, 0x03, 0x21, 0xd7, 0x07, 0x16, 0x16, 0x2b, 0x6b, 0xf8, 0xd9, 0xf5, 0x89, 0xca, 0x33, 0xae, 0x6e, 0x32, 0xb3,
        0xb1, 0x9a, 0xbb, 0x7f, 0xa6, 0x5d, 0x11, 0x41, 0xa3, 0xf9, 0x4d,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 0x11e9db49828,
    // bits: 525264379, // Golang devnet genesis bits
    bits: 0x1e21bc1c, // Bits with ~testnet-like difficulty for slow devnet start
    nonce: 0x48e5e,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00, // Script version
        0x01,                                                                   // Varint
        0x00,                                                                   // OP-FALSE
        0x6b, 0x61, 0x73, 0x70, 0x61, 0x2d, 0x64, 0x65, 0x76, 0x6e, 0x65, 0x74, // vecno-devnet
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::bps::Testnet11Bps, merkle::calc_hash_merkle_root};

    #[test]
    fn test_genesis_hashes() {
        [GENESIS, TESTNET_GENESIS, SIMNET_GENESIS, DEVNET_GENESIS].into_iter().for_each(|genesis| {
            let block: Block = (&genesis).into();
            assert_hashes_eq(calc_hash_merkle_root(block.transactions.iter(), false), block.header.hash_merkle_root);
            assert_hashes_eq(block.hash(), genesis.hash);
        });
    }

    #[test]

    fn assert_hashes_eq(got: Hash, expected: Hash) {
        if got != expected {
            // Special hex print to ease changing the genesis hash according to the print if needed
            panic!("Got hash {:#04x?} while expecting {:#04x?}", got.as_bytes(), expected.as_bytes());
        }
    }
}
