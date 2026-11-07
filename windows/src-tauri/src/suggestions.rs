use crate::Result;
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap,sync::{Mutex,OnceLock},time::{Duration,Instant}};
type Cache=HashMap<String,(Instant,Vec<String>)>;
static CACHE:OnceLock<Mutex<Cache>>=OnceLock::new();

pub fn parse(value:&Value)->Result<Vec<String>>{
 if value["code"].as_i64()!=Some(0){return Err("搜索联想暂不可用".into())}
 let items=value["result"]["tag"].as_array().ok_or("搜索联想返回格式异常")?;
 let mut out=Vec::new();for item in items{if let Some(text)=item["value"].as_str().or_else(||item["term"].as_str()){let text=text.trim();if !text.is_empty()&&text.len()<500&&!out.iter().any(|v|v==text){out.push(text.to_string());if out.len()==8{break}}}}Ok(out)
}
#[tauri::command]
pub async fn search_suggestions(keyword:String)->Result<Vec<String>>{
 let key=keyword.trim().to_string();if key.is_empty()||key.len()>200{return Ok(Vec::new())}
 if key.starts_with("http://")||key.starts_with("https://")||key.starts_with("BV"){return Ok(Vec::new())}
 let cache=CACHE.get_or_init(||Mutex::new(HashMap::new()));
 if let Some((time,items))=cache.lock().map_err(|_|"搜索联想缓存不可用")?.get(&key){if time.elapsed()<Duration::from_secs(120){return Ok(items.clone())}}
 // Public suggestion endpoint; no login cookies or tokens are sent.
 let http=Client::builder().timeout(Duration::from_secs(5)).user_agent("Mozilla/5.0").build().map_err(|_|"无法连接搜索联想服务")?;
 let response=http.get("https://s.search.bilibili.com/main/suggest").query(&[("term",key.as_str()),("main_ver","v1"),("func","suggest"),("suggest_type","accurate"),("sub_type","tag"),("tag_num","8"),("special_acc_num","0")]).send().await.map_err(|_|"搜索联想连接超时")?.error_for_status().map_err(|_|"搜索联想暂不可用")?;
 let data:Value=response.json().await.map_err(|_|"搜索联想返回格式异常")?;let items=parse(&data)?;
 let mut saved=cache.lock().map_err(|_|"搜索联想缓存不可用")?;saved.retain(|_,(time,_)|time.elapsed()<Duration::from_secs(120));if saved.len()>100{saved.clear()}saved.insert(key,(Instant::now(),items.clone()));Ok(items)
}
