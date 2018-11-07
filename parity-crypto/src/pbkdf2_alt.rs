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

extern crate pbkdf2;
extern crate hmac;
use self::hmac::Hmac;

use rsha2::{Sha256, Sha512};

pub struct Salt<'a>(pub &'a [u8]);
pub struct Secret<'a>(pub &'a [u8]);

pub fn sha256(iter: u32, salt: Salt, sec: Secret, out: &mut [u8; 32]) {
	self::pbkdf2::pbkdf2::<Hmac<Sha256>>(sec.0, salt.0, iter as usize, &mut out[..])
}

pub fn sha512(iter: u32, salt: Salt, sec: Secret, out: &mut [u8; 64]) {
	self::pbkdf2::pbkdf2::<Hmac<Sha512>>(sec.0, salt.0, iter as usize, &mut out[..])
}

#[test]
fn basic_test() {
	let mut dest = [0;32];
	let salt = [5;32];
	let secret = [7;32];
	sha256(3, Salt(&salt[..]), Secret(&secret[..]), &mut dest);
	let res = [242, 33, 31, 124, 36, 223, 179, 185, 206, 175, 190, 253, 85, 33, 23, 126, 141, 29, 23, 97, 66, 63, 51, 196, 27, 255, 135, 206, 74, 137, 172, 87];
	assert_eq!(res, dest);
}
