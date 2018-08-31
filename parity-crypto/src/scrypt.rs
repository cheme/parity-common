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

use error::ScryptError;
use super::{KEY_LENGTH_AES, KEY_LENGTH};
use rscrypt:: {scrypt, ScryptParams};
#[cfg(test)]
use std::io::{ Error };

pub fn derive_key(pass: &[u8], salt: &[u8], n: u32, p: u32, r: u32) -> Result<(Vec<u8>, Vec<u8>), ScryptError> {
	// sanity checks
	let log_n = (32 - n.leading_zeros() - 1) as u8;
	if log_n as u32 >= r * 16 {
		return Err(ScryptError::InvalidN);
	}

	if p as u64 > ((u32::max_value() as u64 - 1) * 32)/(128 * (r as u64)) {
		return Err(ScryptError::InvalidP);
	}

	let mut derived_key = vec![0u8; KEY_LENGTH];
	let scrypt_params = ScryptParams::new(log_n, r, p)?;
	scrypt(pass, salt, &scrypt_params, &mut derived_key)?;
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES..KEY_LENGTH];
	Ok((derived_right_bits.to_vec(), derived_left_bits.to_vec()))
}


// test is build from previous crypto lib behaviour, values may be incorrect
// if previous crypto lib got a bug.
#[test]
pub fn test_derive() -> Result<(),Error> {
  let pass = include_bytes!("../test/pass1");
  let salt_v = include_bytes!("../test/salt1");
  let mut salt = [0;32];
  salt.copy_from_slice(&salt_v[..32]);
  let r1 = include_bytes!("../test/right1_1");
  let r2 = include_bytes!("../test/right1_2");
  let l1 = include_bytes!("../test/left1_1");
  let l2 = include_bytes!("../test/left1_2");

  let (l,r) = derive_key(&pass[..],&salt, 262, 1, 8).unwrap();
  assert!(l == r1);
  assert!(r == l1);
  let (l,r) = derive_key(&pass[..],&salt, 144, 4, 4).unwrap();
  assert!(l == r2);
  assert!(r == l2);
  Ok(())
}
