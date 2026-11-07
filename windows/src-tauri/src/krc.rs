use crate::Result;
use base64::{engine::general_purpose::STANDARD,Engine};
use std::io::Read;

pub fn decode(encoded:&str)->Result<String>{
 if encoded.len()>2_000_000{return Err("逐字歌词数据过大".into())}
 let bytes=STANDARD.decode(encoded).map_err(|_|"逐字歌词编码无效")?;
 decode_bytes(&bytes)
}
pub fn decode_bytes(bytes:&[u8])->Result<String>{
 if bytes.len()>2_000_000{return Err("逐字歌词文件过大".into())}
 if !bytes.starts_with(b"krc1"){return String::from_utf8(bytes.to_vec()).map_err(|_|"逐字歌词格式无效".into())}
 // KRC's public file format: krc1 header, repeating XOR mask, zlib payload.
 let key:[u8;16]=[64,71,97,119,94,50,116,71,81,54,49,45,206,210,110,105];
 let payload:Vec<u8>=bytes[4..].iter().enumerate().map(|(i,b)|b^key[i%key.len()]).collect();
 let mut decoder=flate2::read::ZlibDecoder::new(payload.as_slice()).take(2_000_001);let mut text=String::new();decoder.read_to_string(&mut text).map_err(|_|"逐字歌词解码失败")?;if text.len()>2_000_000{return Err("逐字歌词解压后过大".into())}Ok(text)
}
#[tauri::command]
pub fn decode_lyric_file(bytes:Vec<u8>)->Result<String>{decode_bytes(&bytes)}
