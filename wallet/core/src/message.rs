//!
//! Message signing and verification functions.
//!

use vecno_hashes::{Hash, PersonalMessageSigningHash};
use secp256k1::{Error, XOnlyPublicKey};

/// A personal message (text) that can be signed.
#[derive(Clone)]
pub struct PersonalMessage<'a>(pub &'a str);

impl AsRef<[u8]> for PersonalMessage<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Clone)]
pub struct SignMessageOptions {
    /// The auxiliary randomness exists only to mitigate specific kinds of power analysis
    /// side-channel attacks. Providing it definitely improves security, but omitting it
    /// should not be considered dangerous, as most legacy signature schemes don't provide
    /// mitigations against such attacks. To read more about the relevant discussions that
    /// arose in adding this randomness please see: <https://github.com/sipa/bips/issues/195>
    pub no_aux_rand: bool,
}

/// Sign a message with the given private key
pub fn sign_message(msg: &PersonalMessage, privkey: &[u8; 32], options: &SignMessageOptions) -> Result<Vec<u8>, Error> {
    let hash = calc_personal_message_hash(msg);

    let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice())?;
    let schnorr_key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, privkey)?;

    let sig: [u8; 64] = if options.no_aux_rand {
        *secp256k1::SECP256K1.sign_schnorr_no_aux_rand(&msg, &schnorr_key).as_ref()
    } else {
        *schnorr_key.sign_schnorr(msg).as_ref()
    };

    Ok(sig.to_vec())
}

/// Verifies signed message.
///
/// Produces `Ok(())` if the signature matches the given message and [`secp256k1::Error`]
/// if any of the inputs are incorrect, or the signature is invalid.
///
pub fn verify_message(msg: &PersonalMessage, signature: &Vec<u8>, pubkey: &XOnlyPublicKey) -> Result<(), Error> {
    let hash = calc_personal_message_hash(msg);
    let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice())?;
    let sig = secp256k1::schnorr::Signature::from_slice(signature.as_slice())?;
    sig.verify(&msg, pubkey)
}

fn calc_personal_message_hash(msg: &PersonalMessage) -> Hash {
    let mut hasher = PersonalMessageSigningHash::new();
    hasher.write(msg);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sign message equivalent that's only used for tests
    /// Necessary only because of KIP test vectors
    fn sign_message_with_aux_rand(msg: &PersonalMessage, privkey: &[u8; 32], aux_rand: &[u8; 32]) -> Result<Vec<u8>, Error> {
        let hash = calc_personal_message_hash(msg);

        let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice())?;
        let schnorr_key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, privkey)?;
        let curve = secp256k1::Secp256k1::new();
        let sig: [u8; 64] = *curve.sign_schnorr_with_aux_rand(&msg, &schnorr_key, aux_rand).as_ref();

        Ok(sig.to_vec())
    }

    #[test]
    fn test_basic_sign_and_verify_sign() {
        let pm = PersonalMessage("Hello Vecno!");
        let privkey: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xF9, 0x30, 0x8A, 0x01, 0x92, 0x58, 0xC3, 0x10, 0x49, 0x34, 0x4F, 0x85, 0xF8, 0x9D, 0x52, 0x29, 0xB5, 0x31, 0xC8, 0x45,
            0x83, 0x6F, 0x99, 0xB0, 0x86, 0x01, 0xF1, 0x13, 0xBC, 0xE0, 0x36, 0xF9,
        ])
        .unwrap();

        let sign_with_aux_rand = SignMessageOptions { no_aux_rand: false };
        let sign_with_no_aux_rand = SignMessageOptions { no_aux_rand: true };
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_no_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
    }

    #[test]
    fn test_basic_sign_without_rand_twice_should_get_same_signature() {
        let pm = PersonalMessage("Hello Vecno!");
        let privkey: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
        ];

        let sign_with_no_aux_rand = SignMessageOptions { no_aux_rand: true };
        let signature = sign_message(&pm, &privkey, &sign_with_no_aux_rand).expect("sign_message failed");
        let signature_twice = sign_message(&pm, &privkey, &sign_with_no_aux_rand).expect("sign_message failed");
        assert_eq!(signature, signature_twice);
    }

    #[test]
    fn test_kanji_sign_and_verify_sign() {
        let pm = PersonalMessage("こんにちは世界");
        let privkey: [u8; 32] = [
            0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7, 0x62, 0xE7, 0x16, 0x0F,
            0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xDF, 0xF1, 0xD7, 0x7F, 0x2A, 0x67, 0x1C, 0x5F, 0x36, 0x18, 0x37, 0x26, 0xDB, 0x23, 0x41, 0xBE, 0x58, 0xFE, 0xAE, 0x1D,
            0xA2, 0xDE, 0xCE, 0xD8, 0x43, 0x24, 0x0F, 0x7B, 0x50, 0x2B, 0xA6, 0x59,
        ])
        .unwrap();

        let sign_with_aux_rand = SignMessageOptions { no_aux_rand: false };
        let sign_with_no_aux_rand = SignMessageOptions { no_aux_rand: true };
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_no_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
    }

    #[test]
    fn test_long_text_sign_and_verify_sign() {
        let pm = PersonalMessage("Lorem ipsum dolor sit amet. Aut omnis amet id voluptatem eligendi sit accusantium dolorem 33 corrupti necessitatibus hic consequatur quod et maiores alias non molestias suscipit? Est voluptatem magni qui odit eius est eveniet cupiditate id eius quae aut molestiae nihil eum excepturi voluptatem qui nisi architecto?

Et aliquid ipsa ut quas enim et dolorem deleniti ut eius dicta non praesentium neque est velit numquam. Ut consectetur amet ut error veniam et officia laudantium ea velit nesciunt est explicabo laudantium sit totam aperiam.

Ut omnis magnam et accusamus earum rem impedit provident eum commodi repellat qui dolores quis et voluptate labore et adipisci deleniti. Est nostrum explicabo aut quibusdam labore et molestiae voluptate. Qui omnis nostrum At libero deleniti et quod quia.");
        let privkey: [u8; 32] = [
            0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7, 0x62, 0xE7, 0x16, 0x0F,
            0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xDF, 0xF1, 0xD7, 0x7F, 0x2A, 0x67, 0x1C, 0x5F, 0x36, 0x18, 0x37, 0x26, 0xDB, 0x23, 0x41, 0xBE, 0x58, 0xFE, 0xAE, 0x1D,
            0xA2, 0xDE, 0xCE, 0xD8, 0x43, 0x24, 0x0F, 0x7B, 0x50, 0x2B, 0xA6, 0x59,
        ])
        .unwrap();

        let sign_with_aux_rand = SignMessageOptions { no_aux_rand: false };
        let sign_with_no_aux_rand = SignMessageOptions { no_aux_rand: true };
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
        verify_message(&pm, &sign_message(&pm, &privkey, &sign_with_no_aux_rand).expect("sign_message failed"), &pubkey)
            .expect("verify_message failed");
    }

    #[test]
    fn test_fail_verify() {
        let pm = PersonalMessage("Not Hello Vecno!");
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xF9, 0x30, 0x8A, 0x01, 0x92, 0x58, 0xC3, 0x10, 0x49, 0x34, 0x4F, 0x85, 0xF8, 0x9D, 0x52, 0x29, 0xB5, 0x31, 0xC8, 0x45,
            0x83, 0x6F, 0x99, 0xB0, 0x86, 0x01, 0xF1, 0x13, 0xBC, 0xE0, 0x36, 0xF9,
        ])
        .unwrap();
        let fake_sig: Vec<u8> = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
        .to_vec();

        let verify_result = verify_message(&pm, &fake_sig, &pubkey);
        assert!(verify_result.is_err());
    }

    #[test]
    fn test_sign_and_verify_test_case_0() {
        let pm = PersonalMessage("Hello Vecno!");
        let privkey: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
        ];
        let aux_rand: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xF9, 0x30, 0x8A, 0x01, 0x92, 0x58, 0xC3, 0x10, 0x49, 0x34, 0x4F, 0x85, 0xF8, 0x9D, 0x52, 0x29, 0xB5, 0x31, 0xC8, 0x45,
            0x83, 0x6F, 0x99, 0xB0, 0x86, 0x01, 0xF1, 0x13, 0xBC, 0xE0, 0x36, 0xF9,
        ])
        .unwrap();
        let expected_sig: Vec<u8> = [
            0x40, 0xB9, 0xBB, 0x2B, 0xE0, 0xAE, 0x02, 0x60, 0x72, 0x79, 0xED, 0xA6, 0x40, 0x15, 0xA8, 0xD8, 0x6E, 0x37, 0x63, 0x27,
            0x91, 0x70, 0x34, 0x0B, 0x82, 0x43, 0xF7, 0xCE, 0x53, 0x44, 0xD7, 0x7A, 0xFF, 0x11, 0x91, 0x59, 0x8B, 0xAF, 0x2F, 0xD2,
            0x61, 0x49, 0xCA, 0xC3, 0xB4, 0xB1, 0x2C, 0x2C, 0x43, 0x32, 0x61, 0xC0, 0x08, 0x34, 0xDB, 0x60, 0x98, 0xCB, 0x17, 0x2A,
            0xA4, 0x8E, 0xF5, 0x22,
        ]
        .to_vec();

        let sig_result = sign_message_with_aux_rand(&pm, &privkey, &aux_rand).expect("sign_message failed");
        assert_eq!(expected_sig, sig_result);

        verify_message(&pm, &sig_result, &pubkey).expect("verify_message failed");
    }

    #[test]
    fn test_sign_and_verify_test_case_1() {
        let pm = PersonalMessage("Hello Vecno!");
        let privkey: [u8; 32] = [
            0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7, 0x62, 0xE7, 0x16, 0x0F,
            0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
        ];
        let aux_rand: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xDF, 0xF1, 0xD7, 0x7F, 0x2A, 0x67, 0x1C, 0x5F, 0x36, 0x18, 0x37, 0x26, 0xDB, 0x23, 0x41, 0xBE, 0x58, 0xFE, 0xAE, 0x1D,
            0xA2, 0xDE, 0xCE, 0xD8, 0x43, 0x24, 0x0F, 0x7B, 0x50, 0x2B, 0xA6, 0x59,
        ])
        .unwrap();
        let expected_sig: Vec<u8> = [
            0xEB, 0x9E, 0x8A, 0x3C, 0x54, 0x7E, 0xB9, 0x1B, 0x6A, 0x75, 0x92, 0x64, 0x4F, 0x32, 0x8F, 0x06, 0x48, 0xBD, 0xD2, 0x1A,
            0xBA, 0x3C, 0xD4, 0x47, 0x87, 0xD4, 0x29, 0xD4, 0xD7, 0x90, 0xAA, 0x8B, 0x96, 0x27, 0x45, 0x69, 0x1F, 0x3B, 0x47, 0x2E,
            0xD8, 0xD6, 0x5F, 0x3B, 0x77, 0x0E, 0xCB, 0x4F, 0x77, 0x7B, 0xD1, 0x7B, 0x1D, 0x30, 0x91, 0x00, 0x91, 0x9B, 0x53, 0xE0,
            0xE2, 0x06, 0xB4, 0xC6,
        ]
        .to_vec();

        let sig_result = sign_message_with_aux_rand(&pm, &privkey, &aux_rand).expect("sign_message failed");
        assert_eq!(expected_sig, sig_result);

        verify_message(&pm, &sig_result, &pubkey).expect("verify_message failed");
    }

    #[test]
    fn test_sign_and_verify_test_case_2() {
        let pm = PersonalMessage("こんにちは世界");
        let privkey: [u8; 32] = [
            0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7, 0x62, 0xE7, 0x16, 0x0F,
            0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
        ];
        let aux_rand: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xDF, 0xF1, 0xD7, 0x7F, 0x2A, 0x67, 0x1C, 0x5F, 0x36, 0x18, 0x37, 0x26, 0xDB, 0x23, 0x41, 0xBE, 0x58, 0xFE, 0xAE, 0x1D,
            0xA2, 0xDE, 0xCE, 0xD8, 0x43, 0x24, 0x0F, 0x7B, 0x50, 0x2B, 0xA6, 0x59,
        ])
        .unwrap();
        let expected_sig: Vec<u8> = [
            0x81, 0x06, 0x53, 0xD5, 0xF8, 0x02, 0x06, 0xDB, 0x51, 0x96, 0x72, 0x36, 0x2A, 0xDD, 0x6C, 0x98, 0xDA, 0xD3, 0x78, 0x84,
            0x4E, 0x5B, 0xA4, 0xD8, 0x9A, 0x22, 0xC9, 0xF0, 0xC7, 0x09, 0x2E, 0x8C, 0xEC, 0xBA, 0x73, 0x4F, 0xFF, 0x79, 0x22, 0xB6,
            0x56, 0xB4, 0xBE, 0x3F, 0x4B, 0x1F, 0x09, 0x88, 0x99, 0xC9, 0x5C, 0xB5, 0xC1, 0x02, 0x3D, 0xCE, 0x35, 0x19, 0x20, 0x8A,
            0xFA, 0xFB, 0x59, 0xBC,
        ]
        .to_vec();

        let sig_result = sign_message_with_aux_rand(&pm, &privkey, &aux_rand).expect("sign_message failed");
        assert_eq!(expected_sig, sig_result);

        verify_message(&pm, &sig_result, &pubkey).expect("verify_message failed");
    }

    #[test]
    fn test_sign_and_verify_test_case_3() {
        let pm = PersonalMessage("Lorem ipsum dolor sit amet. Aut omnis amet id voluptatem eligendi sit accusantium dolorem 33 corrupti necessitatibus hic consequatur quod et maiores alias non molestias suscipit? Est voluptatem magni qui odit eius est eveniet cupiditate id eius quae aut molestiae nihil eum excepturi voluptatem qui nisi architecto?

Et aliquid ipsa ut quas enim et dolorem deleniti ut eius dicta non praesentium neque est velit numquam. Ut consectetur amet ut error veniam et officia laudantium ea velit nesciunt est explicabo laudantium sit totam aperiam.

Ut omnis magnam et accusamus earum rem impedit provident eum commodi repellat qui dolores quis et voluptate labore et adipisci deleniti. Est nostrum explicabo aut quibusdam labore et molestiae voluptate. Qui omnis nostrum At libero deleniti et quod quia.");
        let privkey: [u8; 32] = [
            0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7, 0x62, 0xE7, 0x16, 0x0F,
            0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
        ];
        let aux_rand: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let pubkey = XOnlyPublicKey::from_slice(&[
            0xDF, 0xF1, 0xD7, 0x7F, 0x2A, 0x67, 0x1C, 0x5F, 0x36, 0x18, 0x37, 0x26, 0xDB, 0x23, 0x41, 0xBE, 0x58, 0xFE, 0xAE, 0x1D,
            0xA2, 0xDE, 0xCE, 0xD8, 0x43, 0x24, 0x0F, 0x7B, 0x50, 0x2B, 0xA6, 0x59,
        ])
        .unwrap();
        let expected_sig: Vec<u8> = [
            0x40, 0xCB, 0xBD, 0x39, 0x38, 0x86, 0x7B, 0x10, 0x07, 0x6B, 0xB1, 0x48, 0x35, 0x55, 0x7C, 0x06, 0x2F, 0x5B, 0xF6, 0xA4,
            0x68, 0x29, 0x95, 0xFC, 0x8B, 0x0A, 0x1C, 0xD2, 0xED, 0x98, 0x6E, 0xED, 0xAA, 0xA0, 0x0C, 0xFE, 0x04, 0xF6, 0xC9, 0xE5,
            0xA9, 0x54, 0x6B, 0x86, 0x07, 0x32, 0xE5, 0xB9, 0x03, 0xCC, 0x82, 0x78, 0x02, 0x28, 0x64, 0x7D, 0x53, 0x75, 0xBE, 0xC3,
            0xD2, 0xA4, 0x98, 0x3A,
        ]
        .to_vec();

        let sig_result = sign_message_with_aux_rand(&pm, &privkey, &aux_rand).expect("sign_message failed");
        assert_eq!(expected_sig, sig_result);

        verify_message(&pm, &sig_result, &pubkey).expect("verify_message failed");
    }
}
