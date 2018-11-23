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

//! secp256k1 for parity.
//! TODO sized u8 array in proto should be usable if we add methods such as U256 -> &[u8;32] to ethereum_types
//! TODO use SecretKey and PublicKey explicitly in if (with conversion from &[u8]) : methods are
//! highly inefficient here.

extern crate secp256k1;
use clear_on_drop::{clear::Clear, ClearOnDrop};

use super::traits::asym::{SecretKey as SecretKeyTrait, PublicKey as PublicKeyTrait, Asym, FiniteField, FixAsymSharedSecret};

use super::error::Error;

// reexports
pub use self::secp256k1::{
	Error as InnerError,
};

pub use self::secp256k1::key::{SecretKey as SecretKeyInner, PublicKey as PublicKeyInner};
use self::secp256k1::constants::{SECRET_KEY_SIZE, GENERATOR_X, GENERATOR_Y, CURVE_ORDER};

use self::secp256k1::key::{ZERO_KEY as ZERO_BYTES, ONE_KEY as ONE_BYTES};
use self::secp256k1::{
	Message,
	RecoverableSignature,
	RecoveryId,
	ecdh,
};

static MINUS_ONE_BYTES: [u8;32] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 64];

lazy_static! {
	pub static ref SECP256K1: self::secp256k1::Secp256k1<self::secp256k1::All> = self::secp256k1::Secp256k1::new();
	static ref MINUS_ONE_KEY: SecretKey = Secp256k1::secret_from_slice(&MINUS_ONE_BYTES[..]).expect("Init from valid const value; qed");
	static ref ONE_KEY: SecretKey = SecretKey(ONE_BYTES);
	static ref ZERO_KEY: SecretKey = SecretKey(ZERO_BYTES);
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SharedSecretAsRef(pub ecdh::SharedSecret);

impl AsRef<[u8]> for SharedSecretAsRef {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

const SIGN_SIZE: usize = 65;
const PUB_SIZE: usize = 64;
const COMPRESSED_PUB_SIZE: usize = 33;

// not vec size could be reduce to 64 (higher instantiation cost)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PublicKey(PublicKeyInner);

impl PublicKey {

	fn new(inner: PublicKeyInner) -> Self {
		PublicKey(inner)
	}

}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecretKey(pub SecretKeyInner);

impl Drop for SecretKey {
	fn drop(&mut self) {
		let len = self.0.len();
		unsafe {
			let mut v = std::slice::from_raw_parts(self.0.as_mut_ptr(), len);
			Clear::clear(&mut v);
		}
	}
}

impl Asym for Secp256k1 {
	type PublicKey = PublicKey;
	type SecretKey = SecretKey;

	const SIGN_SIZE: usize = SIGN_SIZE;

	/// Warning we use 64 bit pubsize (first bytes of 65 bit representation is 4).
	const PUB_SIZE: usize = PUB_SIZE;

	const SECRET_SIZE: usize = SECRET_KEY_SIZE;

	const KEYPAIR_INPUT_SIZE: usize = Self::SECRET_SIZE;

	fn recover(signature: &[u8], message: &[u8]) -> Result<Self::PublicKey, Error> {
		let context = &SECP256K1;
		let rsig = RecoverableSignature::from_compact(&signature[0..PUB_SIZE], RecoveryId::from_i32(signature[PUB_SIZE] as i32)?)?;
		let pubkey = context.recover(&Message::from_slice(message)?, &rsig)?;
		Ok(PublicKey::new(pubkey))
	}

	/// create a key pair from byte value of the secret key, the calling function is responsible for
	/// erasing the input of memory.
	fn keypair_from_slice(sk_bytes: &[u8]) -> Result<(Self::SecretKey, Self::PublicKey), Error> {
		assert!(sk_bytes.len() == SECRET_KEY_SIZE);
		let sk = SecretKeyInner::from_slice(sk_bytes)?;
		let sc = SecretKey(sk);
		let pk = PublicKeyInner::from_secret_key(&SECP256K1, &sc.0);
		Ok((sc, PublicKey::new(pk)))
	}

	fn public_from_secret(s: &Self::SecretKey) -> Result<Self::PublicKey, Error> {
		Ok(PublicKey::new(PublicKeyInner::from_secret_key(&SECP256K1, &s.0)))
	}

	/// using a shortened 64bit public key as input
	fn public_from_slice(public_sec_raw: &[u8]) -> Result<Self::PublicKey, Error> {
		let pdata = {
			let mut temp = [4u8; PUB_SIZE + 1];
			(&mut temp[1..PUB_SIZE + 1]).copy_from_slice(&public_sec_raw[0..PUB_SIZE]);
			temp
		};
		Ok(PublicKey::new(PublicKeyInner::from_slice(&pdata)?))
	}

	fn secret_from_slice(secret: &[u8]) -> Result<Self::SecretKey, Error> {
		Ok(SecretKey(SecretKeyInner::from_slice(secret)?))
	}

}

impl FixAsymSharedSecret for SecretKey {
	type Other = PublicKey;
	type Result = SharedSecretAsRef;

	fn shared_secret(&self, publ: &Self::Other) -> Result<Self::Result, Error> {
		let shared = ecdh::SharedSecret::new_raw(&publ.0, &self.0);
		Ok(SharedSecretAsRef(shared))
	}

}

impl SecretKeyTrait for SecretKey {
	type VecRepr = ClearOnDrop<Vec<u8>>;

	fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
		let context = &SECP256K1;
		let s = context.sign_recoverable(&Message::from_slice(message)?, &self.0);
		let (rec_id, data) = s.serialize_compact();
		let mut data_arr = vec![0; SIGN_SIZE];

		// no need to check if s is low, it always is
		data_arr[0..PUB_SIZE].copy_from_slice(&data[0..PUB_SIZE]);
		data_arr[PUB_SIZE] = rec_id.to_i32() as u8;
		Ok(data_arr)
	}

	fn to_vec(&self) -> Self::VecRepr {
		ClearOnDrop::new(self.0[..].to_vec())
	}

}

impl PublicKeyTrait for PublicKey {
	const COMPRESSED_PUB_SIZE: usize = COMPRESSED_PUB_SIZE;
	type VecRepr = Vec<u8>;
	type CompVecRepr = Vec<u8>;

	fn to_vec(&self) -> Self::VecRepr {
		self.0.serialize_uncompressed()[1..PUB_SIZE + 1].to_vec()
	}

	/// Should move to another trait.
	fn to_compressed_vec(&self) -> Self::CompVecRepr {
		self.0.serialize().to_vec()
	}

	fn verify(&self, signature: &[u8], message: &[u8]) -> Result<bool, Error> {
		let context = &SECP256K1;
		let rsig = RecoverableSignature::from_compact(&signature[0..PUB_SIZE], RecoveryId::from_i32(signature[PUB_SIZE] as i32)?)?;
		let sig = rsig.to_standard();

		match context.verify(&Message::from_slice(message)?, &sig, &self.0) {
			Ok(_) => Ok(true),
			Err(InnerError::IncorrectSignature) => Ok(false),
			Err(x) => Err(InnerError::from(x).into())
		}
	}

}

pub struct Secp256k1;

impl FiniteField for Secp256k1 {

	fn generator_x() -> &'static[u8] { &GENERATOR_X[..] }
	fn generator_y() -> &'static[u8] { &GENERATOR_Y[..] }
	fn curve_order() -> &'static[u8] { &CURVE_ORDER[..] }

	fn public_mul(pub_key: &mut Self::PublicKey, sec_key: &Self::SecretKey) -> Result<(), Error> {
		pub_key.0.mul_assign(&SECP256K1, &sec_key.0)?;
		Ok(())
	}

	fn public_add(pub_key: &mut Self::PublicKey, other_public: &Self::PublicKey) -> Result<(), Error> {
		let res = other_public.0.combine(&pub_key.0)?;
		*pub_key = PublicKey::new(res);
		Ok(())
	}

	fn secret_mul(sec_key: &mut Self::SecretKey, other_secret: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.mul_assign(&other_secret.0)?;
		Ok(())
	}

	fn secret_add(sec_key: &mut Self::SecretKey, other_secret: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.add_assign(&other_secret.0)?;
		Ok(())
	}

	fn secret_inv(sec_key: &mut Self::SecretKey) -> Result<(), Error> {
		sec_key.0.inv_assign()?;
		Ok(())
	}

	fn one_key() -> &'static Self::SecretKey {
		&ONE_KEY
	}

	fn zero_key() -> &'static Self::SecretKey {
		&ZERO_KEY
	}

	fn minus_one_key() -> &'static Self::SecretKey {
		&MINUS_ONE_KEY
	}

}

impl From<InnerError> for Error {
	fn from(err: InnerError) -> Self {
		match err {
			InnerError::InvalidSecretKey => Error::AsymShort("Invalid secret"),
			InnerError::InvalidRecoveryId => Error::AsymShort("Invalid recovery id"),
			InnerError::InvalidPublicKey => Error::AsymShort("Invalid public"),
			InnerError::InvalidSignature |
			InnerError::IncorrectSignature => Error::AsymShort("Invalid EC signature"),
			InnerError::InvalidMessage => Error::AsymShort("Invalid AES message"),
		}
	}
}

#[cfg(test)]
type AsymTest = Secp256k1;

#[cfg(test)]
::tests_asym!();
