// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A key-value database for use in browsers
//!
//! Writes data both into memory and IndexedDB, reads the whole database in memory
//! from the IndexedDB on `open`.

#![deny(missing_docs)]

mod error;
mod indexed_db;

use kvdb::{DBTransaction, DBValue};
use kvdb_memorydb::{self as in_memory, InMemory};
use send_wrapper::SendWrapper;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use log::{debug, warn};

pub use error::Error;
pub use kvdb::KeyValueDB;

use futures::prelude::*;

use web_sys::IdbDatabase;

/// Database backed by both IndexedDB and in memory implementation.
pub struct Database {
	name: String,
	version: u32,
	columns: u32,
	in_memory: InMemory,
	indexed_db: SendWrapper<IdbDatabase>,
	// Note that this limit to a single write thread to indexed_db
	synch: Arc<AtomicBool>,
}

// TODO: implement when web-based implementation need memory stats
parity_util_mem::malloc_size_of_is_0!(Database);

impl Database {
	/// Opens the database with the given name,
	/// and the specified number of columns (not including the default one).
	pub async fn open(name: String, columns: u32) -> Result<Database, error::Error> {
		let name_clone = name.clone();
		// let's try to open the latest version of the db first
		let db = indexed_db::open(name.as_str(), None, columns).await?;

		// If we need more column than the latest version has,
		// then bump the version (+ 1 for the default column).
		// In order to bump the version, we close the database
		// and reopen it with a higher version than it was opened with previously.
		// cf. https://github.com/paritytech/parity-common/pull/202#discussion_r321221751
		let db = if columns + 1 > db.columns {
			let next_version = db.version + 1;
			drop(db);
			indexed_db::open(name.as_str(), Some(next_version), columns).await?
		} else {
			db
		};
		// populate the in_memory db from the IndexedDB
		let indexed_db::IndexedDB { version, inner, .. } = db;
		let in_memory = in_memory::create(columns);
		// read the columns from the IndexedDB
		for column in 0..columns {
			let mut nb_kv = 0usize;
			let mut size_w = 0usize;
			let mut key_size_w = 0usize;
			let nb_kv = &mut nb_kv;
			let size_w = &mut size_w;
			let key_size_w = &mut key_size_w;

			let mut txn = DBTransaction::new();
			let mut stream = indexed_db::idb_cursor(&*inner, column);
			while let Some((key, value)) = stream.next().await {
				*nb_kv += 1;
				*size_w += value.len();
				*key_size_w += key.as_slice().len();
				txn.put_vec(column, key.as_ref(), value);
			}
			warn!("loaded col_{}: {} {} {}", column, *nb_kv, *key_size_w, *size_w);
			// write each column into memory
			in_memory.write(txn).expect("writing in memory always succeeds; qed");
		}
		Ok(Database {
			name: name_clone,
			version,
			columns,
			in_memory,
			indexed_db: inner,
			synch: Arc::new(AtomicBool::new(false)),
		})
	}

	/// Get the database name.
	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	/// Get the database version.
	pub fn version(&self) -> u32 {
		self.version
	}
}

impl Drop for Database {
	fn drop(&mut self) {
		self.indexed_db.close();
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>> {
		self.in_memory.get(col, key)
	}

	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.in_memory.get_by_prefix(col, prefix)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		self.synch.store(true, Ordering::SeqCst);
		let _ = indexed_db::idb_commit_transaction(&*self.indexed_db, &transaction, self.columns, self.synch.clone());
		while self.synch.load(Ordering::SeqCst) {
			// TODO some incremental calls to js timeout (indexeddb run in a different thread
		}
		self.in_memory.write(transaction)
	}

	// NOTE: clones the whole db
	fn iter<'a>(&'a self, col: u32) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
		self.in_memory.iter(col)
	}

	// NOTE: clones the whole db
	fn iter_with_prefix<'a>(
		&'a self,
		col: u32,
		prefix: &'a [u8],
	) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
		self.in_memory.iter_with_prefix(col, prefix)
	}

	// NOTE: not supported
	fn restore(&self, _new_db: &str) -> std::io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Other, "Not supported yet"))
	}
}
