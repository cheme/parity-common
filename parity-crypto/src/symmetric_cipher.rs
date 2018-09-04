// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Trait Api used by parity for symmetric cipher.

quick_error! {
  /// warn do not include error from other crates
    #[derive(Debug)]
    pub enum SymmError {
        OtherError(e: String) {
            display("Other Error : {}", e)
            from()
        }
    }
}

type Result<T> = ::std::result::Result<T, SymmError>;

//------------low level

trait Encrypter: Crypter {
    fn encrypt(&mut self, content: &mut [u8]) -> Result<()>;
}

trait PaddedEncrypter: Crypter {
    fn encrypt(&mut self, content: &mut [u8]) -> Result<()>;

    fn finalize(&mut self, dest: &mut [u8]) -> Result<()>;
}

trait Decrypter: Crypter {
    fn decrypt(&mut self, content: &mut [u8]) -> Result<usize>;
}

trait Cipher {

    type E: Encrypter;
    type D: Decrypter;

    /// having separate trait when no iv is needed makes also sense, for now keep it simple
    fn new_enc(key: &[u8], iv: Option<&[u8]>) -> Result<Self::E>;
    fn new_dec(key: &[u8], iv: Option<&[u8]>) -> Result<Self::D>;
}

trait Crypter: Sized {

    fn key_len() -> usize;
    /// when none no iv, when some allways iv
    fn iv_len() -> Option<usize>;
    fn block_len() -> usize;

    /// TODO clean comment, use it to restate afresh a crypto cipher
    /// Note that new_iv is confusing : must use an iv for cipher with iv.
    fn clear_state(self, new_iv: Option<&[u8]>) -> Result<Self>;

    // usefull ??
    // fn get_key(&self) -> Result<Vec<u8>>;
}

#[cfg(feature = "rustCrypto")]
pub mod rustCrypto {
    extern crate block_cipher_trait;
    use self::block_cipher_trait::{BlockCipher, InvalidKeyLength};
    use super::*;
    use aes_ctr::stream_cipher::{LoopError, NewFixStreamCipher, StreamCipherCore};
    use aes_ctr::{Aes128Ctr, Aes256Ctr};
    use block_modes::block_padding::Pkcs7;
    use block_modes::block_padding::ZeroPadding;
    use block_modes::{BlockMode, BlockModeIv};
    use block_modes::{BlockModeError, Cbc, Ecb};
    use raes::block_cipher_trait::generic_array::typenum::Unsigned;
    use raes::block_cipher_trait::generic_array::GenericArray;
    use raes::{Aes128, Aes256};

    impl From<BlockModeError> for SymmError {
        fn from(e: BlockModeError) -> SymmError {
            SymmError::OtherError("BlockModeError".to_string()) // TODO define generic error type
        }
    }

    impl From<InvalidKeyLength> for SymmError {
        fn from(e: InvalidKeyLength) -> SymmError {
            SymmError::OtherError("InvalidKeyLength".to_string()) // TODO define generic error type
        }
    }

    impl From<LoopError> for SymmError {
        fn from(e: LoopError) -> SymmError {
            SymmError::OtherError(e.to_string()) // TODO define generic error type
        }
    }

    pub struct AesEcb256;

    pub struct AesEcb256_cr(Ecb<Aes256, ZeroPadding>);

    impl Cipher for AesEcb256 {
        type E = AesEcb256_cr;
        type D = AesEcb256_cr;

        fn new_enc(key: &[u8], iv: Option<&[u8]>) -> Result<Self::E> {
            assert!(iv.is_none());
            Ok(AesEcb256_cr(Ecb::new_varkey(key)?))
        }
        fn new_dec(key: &[u8], iv: Option<&[u8]>) -> Result<Self::D> {
            assert!(iv.is_none());
            Ok(AesEcb256_cr(Ecb::new_varkey(key)?))
        }
    }

    impl Crypter for AesEcb256_cr {
        fn key_len() -> usize {
            <Aes256 as BlockCipher>::KeySize::to_usize()
        }
        fn block_len() -> usize {
            <Aes256 as BlockCipher>::BlockSize::to_usize()
        }
        fn iv_len() -> Option<usize> {
            None
        }

        fn clear_state(self, _new_iv: Option<&[u8]>) -> Result<Self> {
            Ok(self)
        }
    }

    impl Encrypter for AesEcb256_cr {
        fn encrypt(&mut self, content: &mut [u8]) -> Result<()> {
            self.0.encrypt_nopad(content)?;
            Ok(())
        }
    }

    impl Decrypter for AesEcb256_cr {
        fn decrypt(&mut self, content: &mut [u8]) -> Result<usize> {
            self.0.decrypt_nopad(content)?;
            Ok(content.len())
        }
    }

    pub struct AesCtr256;

    // TODO probably a mis used of aes ctr : should be stream level abstraction  -> need testing!!
    pub struct AesCtr256_cr(Aes256Ctr);

    impl Cipher for AesCtr256 {
        type E = AesCtr256_cr;
        type D = AesCtr256_cr;

        fn new_enc(key: &[u8], iv: Option<&[u8]>) -> Result<Self::E> {
            Ok(AesCtr256_cr(Aes256Ctr::new(
                GenericArray::from_slice(key),
                GenericArray::from_slice(iv.expect("ctr need iv")),
            )))
        }
        fn new_dec(key: &[u8], iv: Option<&[u8]>) -> Result<Self::D> {
            Self::new_enc(key, iv)
        }
    }
    impl Crypter for AesCtr256_cr {
        fn key_len() -> usize {
            <Aes256 as BlockCipher>::KeySize::to_usize()
        }
        fn block_len() -> usize {
            <Aes256 as BlockCipher>::BlockSize::to_usize()
        }
        fn iv_len() -> Option<usize> {
            Some(<Aes256 as BlockCipher>::BlockSize::to_usize())
        }

        fn clear_state(self, new_iv: Option<&[u8]>) -> Result<Self> {
            // Damn require key storage
            panic!("unimplemented");
        }
    }
    impl Encrypter for AesCtr256_cr {
        fn encrypt(&mut self, content: &mut [u8]) -> Result<()> {
            self.0.try_apply_keystream(content)?;
            Ok(())
        }
    }

    impl Decrypter for AesCtr256_cr {
        fn decrypt(&mut self, content: &mut [u8]) -> Result<usize> {
            self.0.try_apply_keystream(content)?;
            Ok(content.len())
        }
    }

}
