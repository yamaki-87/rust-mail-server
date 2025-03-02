use anyhow::Result;
use base64::{engine::general_purpose, Engine};

/// ## Summary
/// base64文字をdecodeする
///
/// ## Parameters
/// - `input`: base64文字
///
/// ## Returns
/// utf8にdeocdeされた文字
///
/// ## Examples
///```
///
///```
pub fn decode(input: &str) -> Result<String> {
    let bytes = deocde_bytes(input)?;

    Ok(String::from_utf8(bytes)?)
}

pub fn deocde_bytes(input: &str) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(input.trim())?)
}
