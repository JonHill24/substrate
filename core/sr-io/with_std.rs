// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use primitives::{
	blake2_256, twox_128, twox_256, ed25519, Blake2Hasher, sr25519,
	Pair
};
// Switch to this after PoC-3
// pub use primitives::BlakeHasher;
pub use substrate_state_machine::{Externalities, BasicExternalities, TestExternalities};

use environmental::environmental;
use primitives::{hexdisplay::HexDisplay, H256};

#[cfg(feature = "std")]
use std::collections::HashMap;

environmental!(ext: trait Externalities<Blake2Hasher>);

/// Additional bounds for `Hasher` trait for with_std.
pub trait HasherBounds {}
impl<T: Hasher> HasherBounds for T {}

impl StorageApi for () {
	fn storage(key: &[u8]) -> Option<Vec<u8>> {
		ext::with(|ext| ext.storage(key).map(|s| s.to_vec()))
			.expect("storage cannot be called outside of an Externalities-provided environment.")
	}

	fn child_storage(storage_key: &[u8], key: &[u8]) -> Option<Vec<u8>> {
		ext::with(|ext| ext.child_storage(storage_key, key).map(|s| s.to_vec()))
			.expect("storage cannot be called outside of an Externalities-provided environment.")
	}

	fn read_storage(key: &[u8], value_out: &mut [u8], value_offset: usize) -> Option<usize> {
		ext::with(|ext| ext.storage(key).map(|value| {
			let value = &value[value_offset..];
			let written = ::std::cmp::min(value.len(), value_out.len());
			value_out[..written].copy_from_slice(&value[..written]);
			value.len()
		})).expect("read_storage cannot be called outside of an Externalities-provided environment.")
	}

	fn read_child_storage(storage_key: &[u8], key: &[u8], value_out: &mut [u8], value_offset: usize) -> Option<usize> {
		ext::with(|ext| ext.child_storage(storage_key, key).map(|value| {
			let value = &value[value_offset..];
			let written = ::std::cmp::min(value.len(), value_out.len());
			value_out[..written].copy_from_slice(&value[..written]);
			value.len()
		})).expect("read_storage cannot be called outside of an Externalities-provided environment.")
	}

	fn set_storage(key: &[u8], value: &[u8]) {
		ext::with(|ext|
			ext.set_storage(key.to_vec(), value.to_vec())
		);
	}

	fn set_child_storage(storage_key: &[u8], key: &[u8], value: &[u8]) {
		ext::with(|ext|
			ext.set_child_storage(storage_key.to_vec(), key.to_vec(), value.to_vec())
		);
	}

	fn clear_storage(key: &[u8]) {
		ext::with(|ext|
			ext.clear_storage(key)
		);
	}

	fn clear_child_storage(storage_key: &[u8], key: &[u8]) {
		ext::with(|ext|
			ext.clear_child_storage(storage_key, key)
		);
	}

	fn kill_child_storage(storage_key: &[u8]) {
		ext::with(|ext|
			ext.kill_child_storage(storage_key)
		);
	}

	fn exists_storage(key: &[u8]) -> bool {
		ext::with(|ext|
			ext.exists_storage(key)
		).unwrap_or(false)
	}

	fn exists_child_storage(storage_key: &[u8], key: &[u8]) -> bool {
		ext::with(|ext|
			ext.exists_child_storage(storage_key, key)
		).unwrap_or(false)
	}

	fn clear_prefix(prefix: &[u8]) {
		ext::with(|ext|
			ext.clear_prefix(prefix)
		);
	}

	fn storage_root() -> [u8; 32] {
		ext::with(|ext|
			ext.storage_root()
		).unwrap_or(H256::zero()).into()
	}

	fn child_storage_root(storage_key: &[u8]) -> Option<Vec<u8>> {
		ext::with(|ext|
			ext.child_storage_root(storage_key)
		).unwrap_or(None)
	}

	fn storage_changes_root(parent_hash: [u8; 32], parent_num: u64) -> Option<[u8; 32]> {
		ext::with(|ext|
			ext.storage_changes_root(parent_hash.into(), parent_num).map(Into::into)
		).unwrap_or(None)
	}

	fn enumerated_trie_root<H>(input: &[&[u8]]) -> H::Out
	where
		H: Hasher,
		H::Out: Ord,
	{
		trie::ordered_trie_root::<H, _, _>(input.iter())
	}

	fn trie_root<H, I, A, B>(input: I) -> H::Out
	where
		I: IntoIterator<Item = (A, B)>,
		A: AsRef<[u8]> + Ord,
		B: AsRef<[u8]>,
		H: Hasher,
		H::Out: Ord,
	{
		trie::trie_root::<H, _, _, _>(input)
	}

	fn ordered_trie_root<H, I, A>(input: I) -> H::Out
	where
		I: IntoIterator<Item = A>,
		A: AsRef<[u8]>,
		H: Hasher,
		H::Out: Ord,
	{
		trie::ordered_trie_root::<H, _, _>(input)
	}
}

impl OtherApi for () {
	fn chain_id() -> u64 {
		ext::with(|ext|
			ext.chain_id()
		).unwrap_or(0)
	}

	fn print<T: Printable + Sized>(value: T) {
		value.print()
	}
}

impl CryptoApi for () {
	fn ed25519_verify<P: AsRef<[u8]>>(sig: &[u8; 64], msg: &[u8], pubkey: P) -> bool {
		ed25519::Pair::verify_weak(sig, msg, pubkey)
	}

	fn sr25519_verify<P: AsRef<[u8]>>(sig: &[u8; 64], msg: &[u8], pubkey: P) -> bool {
		sr25519::Pair::verify_weak(sig, msg, pubkey)
	}

	fn secp256k1_ecdsa_recover(sig: &[u8; 65], msg: &[u8; 32]) -> Result<[u8; 64], EcdsaVerifyError> {
		let rs = secp256k1::Signature::parse_slice(&sig[0..64]).map_err(|_| EcdsaVerifyError::BadRS)?;
		let v = secp256k1::RecoveryId::parse(if sig[64] > 26 { sig[64] - 27 } else { sig[64] } as u8).map_err(|_| EcdsaVerifyError::BadV)?;
		let pubkey = secp256k1::recover(&secp256k1::Message::parse(msg), &rs, &v).map_err(|_| EcdsaVerifyError::BadSignature)?;
		let mut res = [0u8; 64];
		res.copy_from_slice(&pubkey.serialize()[1..65]);
		Ok(res)
	}
}

impl HashingApi for () {
	fn keccak_256(data: &[u8]) -> [u8; 32] {
		tiny_keccak::keccak256(data)
	}

	fn blake2_256(data: &[u8]) -> [u8; 32] {
		blake2_256(data)
	}

	fn twox_256(data: &[u8]) -> [u8; 32] {
		twox_256(data)
	}

	fn twox_128(data: &[u8]) -> [u8; 16] {
		twox_128(data)
	}
}

impl OffchainApi for () {
	fn submit_extrinsic<T: codec::Encode>(data: &T) {
		ext::with(|ext| ext
			.submit_extrinsic(codec::Encode::encode(data))
			.expect("submit_extrinsic can be called only in offchain worker context")
		).expect("submit_extrinsic cannot be called outside of an Externalities-provided environment.")
	}

	fn timestamp() -> offchain::Timestamp {
		unimplemented!()
	}

	fn http_request_start(
		_method: &str,
		_uri: &str,
		_meta: &[u8]
	) -> offchain::http::RequestId {
		unimplemented!()
	}

	fn http_request_add_header(
		_request_id: offchain::http::RequestId,
		_name: &str,
		_value: &str
	) {
		unimplemented!()
	}

	fn http_request_write_body(
		_request_id: offchain::http::RequestId,
		_chunk: &[u8],
		_deadline: Option<offchain::Timestamp>
	) -> Result<(), ()> {
		unimplemented!()
	}

	fn http_response_wait(
		_ids: &[offchain::http::RequestId],
		_deadline: Option<offchain::Timestamp>
	) -> Vec<offchain::http::RequestStatus> {
		unimplemented!()
	}

	fn http_response_headers(
		_request_id: offchain::http::RequestId
	) -> Vec<(Vec<u8>, Vec<u8>)> {
		unimplemented!()
	}

	fn http_response_read_body(
		_request_id: offchain::http::RequestId,
		_buffer: &mut [u8],
		_deadline: Option<offchain::Timestamp>
	) -> Result<usize, ()> {
		unimplemented!()
	}
}

impl Api for () {}

/// Execute the given closure with global function available whose functionality routes into the
/// externalities `ext`. Forwards the value that the closure returns.
// NOTE: need a concrete hasher here due to limitations of the `environmental!` macro, otherwise a type param would have been fine I think.
pub fn with_externalities<R, F: FnOnce() -> R>(ext: &mut Externalities<Blake2Hasher>, f: F) -> R {
	ext::using(ext, f)
}

/// A set of key value pairs for storage.
pub type StorageOverlay = HashMap<Vec<u8>, Vec<u8>>;

/// A set of key value pairs for children storage;
pub type ChildrenStorageOverlay = HashMap<Vec<u8>, StorageOverlay>;

/// Execute the given closure with global functions available whose functionality routes into
/// externalities that draw from and populate `storage`. Forwards the value that the closure returns.
pub fn with_storage<R, F: FnOnce() -> R>(storage: &mut StorageOverlay, f: F) -> R {
	let mut alt_storage = Default::default();
	rstd::mem::swap(&mut alt_storage, storage);
	let mut ext: BasicExternalities = alt_storage.into();
	let r = ext::using(&mut ext, f);
	*storage = ext.into();
	r
}

impl<'a> Printable for &'a [u8] {
	fn print(self) {
		println!("Runtime: {}", HexDisplay::from(&self));
	}
}

impl<'a> Printable for &'a str {
	fn print(self) {
		println!("Runtime: {}", self);
	}
}

impl Printable for u64 {
	fn print(self) {
		println!("Runtime: {}", self);
	}
}

#[cfg(test)]
mod std_tests {
	use super::*;
	use primitives::map;

	#[test]
	fn storage_works() {
		let mut t = BasicExternalities::default();
		assert!(with_externalities(&mut t, || {
			assert_eq!(storage(b"hello"), None);
			set_storage(b"hello", b"world");
			assert_eq!(storage(b"hello"), Some(b"world".to_vec()));
			assert_eq!(storage(b"foo"), None);
			set_storage(b"foo", &[1, 2, 3][..]);
			true
		}));

		t = BasicExternalities::new(map![b"foo".to_vec() => b"bar".to_vec()]);

		assert!(!with_externalities(&mut t, || {
			assert_eq!(storage(b"hello"), None);
			assert_eq!(storage(b"foo"), Some(b"bar".to_vec()));
			false
		}));
	}

	#[test]
	fn read_storage_works() {
		let mut t = BasicExternalities::new(map![
			b":test".to_vec() => b"\x0b\0\0\0Hello world".to_vec()
		]);

		with_externalities(&mut t, || {
			let mut v = [0u8; 4];
			assert!(read_storage(b":test", &mut v[..], 0).unwrap() >= 4);
			assert_eq!(v, [11u8, 0, 0, 0]);
			let mut w = [0u8; 11];
			assert!(read_storage(b":test", &mut w[..], 4).unwrap() >= 11);
			assert_eq!(&w, b"Hello world");
		});
	}

	#[test]
	fn clear_prefix_works() {
		let mut t = BasicExternalities::new(map![
			b":a".to_vec() => b"\x0b\0\0\0Hello world".to_vec(),
			b":abcd".to_vec() => b"\x0b\0\0\0Hello world".to_vec(),
			b":abc".to_vec() => b"\x0b\0\0\0Hello world".to_vec(),
			b":abdd".to_vec() => b"\x0b\0\0\0Hello world".to_vec()
		]);

		with_externalities(&mut t, || {
			clear_prefix(b":abc");

			assert!(storage(b":a").is_some());
			assert!(storage(b":abdd").is_some());
			assert!(storage(b":abcd").is_none());
			assert!(storage(b":abc").is_none());
		});
	}
}
