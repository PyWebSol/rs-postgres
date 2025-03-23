use crate::data::icons;
use ring::{aead, pbkdf2, rand};
use ring::rand::SecureRandom;
use sha2::Digest;
use std::num::NonZeroU32;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const ITERATION_COUNT: u32 = 100_000;

pub fn load_icon() -> egui::IconData {
	let (icon_rgba, icon_width, icon_height) = {
		let image = image::load_from_memory(icons::RS_POSTGRES_PNG)
			.expect("Failed to open icon path")
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};

	egui::IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}
}

#[derive(Clone, Debug)]
pub struct EncryptedData {
	salt: Vec<u8>,
	nonce: Vec<u8>,
	ciphertext: Vec<u8>,
}

pub fn encrypt_string(plain_text: &str, password: impl ToString) -> Result<String, String> {
	let rng = rand::SystemRandom::new();
	let mut salt = vec![0u8; SALT_LEN];
	rng.fill(&mut salt).map_err(|_| String::from("Error while generating salt"))?;

	let mut nonce = vec![0u8; NONCE_LEN];
	rng.fill(&mut nonce).map_err(|_| String::from("Error while generating nonce"))?;

	let mut key = [0u8; KEY_LEN];
	let iterations = NonZeroU32::new(ITERATION_COUNT).unwrap();
	pbkdf2::derive(
		pbkdf2::PBKDF2_HMAC_SHA256,
		iterations,
		&salt,
		password.to_string().as_bytes(),
		&mut key,
	);

	let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
		.map_err(|_| String::from("Error while creating key"))?;
	let sealing_key = aead::LessSafeKey::new(unbound_key);

	let nonce_sequence = aead::Nonce::assume_unique_for_key(
		nonce.clone().try_into().map_err(|_| String::from("Invalid nonce format"))?
	);

	let mut in_out = plain_text.as_bytes().to_vec();
	sealing_key
		.seal_in_place_append_tag(nonce_sequence, aead::Aad::empty(), &mut in_out)
		.map_err(|_| String::from("Error while encrypting"))?;

	let encrypted_data = EncryptedData {
		salt,
		nonce,
		ciphertext: in_out,
	};

	let serialized = serialize_encrypted_data(&encrypted_data)?;
	Ok(serialized)
}

pub fn decrypt_string(encrypted_text: &str, password: impl ToString) -> Result<String, String> {
	let encrypted_data = deserialize_encrypted_data(encrypted_text)?;

	let mut key = [0u8; KEY_LEN];
	let iterations = NonZeroU32::new(ITERATION_COUNT).unwrap();
	pbkdf2::derive(
		pbkdf2::PBKDF2_HMAC_SHA256,
		iterations,
		&encrypted_data.salt,
		password.to_string().as_bytes(),
		&mut key,
	);

	let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
		.map_err(|_| String::from("Error while creating key"))?;
	let opening_key = aead::LessSafeKey::new(unbound_key);

	let nonce_sequence = aead::Nonce::assume_unique_for_key(
		encrypted_data.nonce.try_into().map_err(|_| String::from("Invalid nonce format"))?
	);

	let mut ciphertext = encrypted_data.ciphertext.clone();
	let plaintext = opening_key
		.open_in_place(nonce_sequence, aead::Aad::empty(), &mut ciphertext)
		.map_err(|_| String::from("Error while decrypting"))?;

	String::from_utf8(plaintext.to_vec())
		.map_err(|_| String::from("Error while decrypting"))
}

fn serialize_encrypted_data(data: &EncryptedData) -> Result<String, String> {
	let mut serialized = Vec::new();

	serialized.extend_from_slice(&(data.salt.len() as u32).to_be_bytes());
	serialized.extend_from_slice(&data.salt);

	serialized.extend_from_slice(&(data.nonce.len() as u32).to_be_bytes());
	serialized.extend_from_slice(&data.nonce);

	serialized.extend_from_slice(&(data.ciphertext.len() as u32).to_be_bytes());
	serialized.extend_from_slice(&data.ciphertext);

	Ok(BASE64.encode(serialized))
}

fn deserialize_encrypted_data(serialized: &str) -> Result<EncryptedData, String> {
	let bytes = BASE64.decode(serialized)
		.map_err(|_| String::from("Error while decoding base64"))?;

	if bytes.len() < 12 {
		return Err(String::from("Not enough data for deserialization"));
	}

	let mut pos = 0;

	let salt_len = u32::from_be_bytes(bytes[pos..pos+4].try_into().unwrap()) as usize;
	pos += 4;
	if pos + salt_len > bytes.len() {
		return Err(String::from("Not enough data for reading salt"));
	}
	let salt = bytes[pos..pos+salt_len].to_vec();
	pos += salt_len;

	if pos + 4 > bytes.len() {
		return Err(String::from("Not enough data for reading nonce size"));
	}
	let nonce_len = u32::from_be_bytes(bytes[pos..pos+4].try_into().unwrap()) as usize;
	pos += 4;
	if pos + nonce_len > bytes.len() {
		return Err(String::from("Not enough data for reading nonce"));
	}
	let nonce = bytes[pos..pos+nonce_len].to_vec();
	pos += nonce_len;

	if pos + 4 > bytes.len() {
		return Err(String::from("Not enough data for reading ciphertext size"));
	}
	let ciphertext_len = u32::from_be_bytes(bytes[pos..pos+4].try_into().unwrap()) as usize;
	pos += 4;
	if pos + ciphertext_len > bytes.len() {
		return Err(String::from("Not enough data for reading ciphertext"));
	}
	let ciphertext = bytes[pos..pos+ciphertext_len].to_vec();

	Ok(EncryptedData {
		salt,
		nonce,
		ciphertext,
	})
}

pub fn create_checksum(text: impl ToString) -> String {
    let mut hasher = sha2::Sha256::new();

    hasher.update(text.to_string());

    format!("{:x}", hasher.finalize())
}
