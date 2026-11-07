use crate::{AppState,Result};
use base64::{engine::general_purpose::STANDARD,Engine};
use reqwest::Client;
use serde_json::{json,Value};
use std::time::Duration;

async fn fetch(http:&Client,title:&str,artist:&str,hash:Option<&str>,duration:u64)->Result<Value>{
 let mut request=http.get("https://lyrics.kugou.com/search").query(&[("ver","1"),("man","yes"),("client","pc"),("keyword",&format!("{artist} {title}")),("timelength",&duration.to_string())]);
 if let Some(hash)=hash.filter(|h|h.len()==32&&h.chars().all(|c|c.is_ascii_hexdigit())){request=request.query(&[("hash",hash)])}
 let response=request.send().await.map_err(|_|"歌词服务连接失败，请稍后重试")?;
 let data:Value=response.error_for_status().map_err(|_|"歌词服务暂不可用")?.json().await.map_err(|_|"歌词服务返回格式异常")?;
 let normalize=|s:&str|s.to_lowercase().chars().filter(|c|c.is_alphanumeric()).collect::<String>();
 let title_key=normalize(title);let artist_key=normalize(artist);
 if let Some(items)=data["candidates"].as_array(){
  let candidate=items.iter().filter(|x|x["id"].is_string()&&x["accesskey"].is_string()).filter(|x|normalize(x["song"].as_str().unwrap_or(""))==title_key)
   .filter(|x|artist_key.is_empty()||{let singer=normalize(x["singer"].as_str().unwrap_or(""));!singer.is_empty()&&(singer.contains(&artist_key)||artist_key.contains(&singer))})
   .min_by_key(|x|x["duration"].as_u64().unwrap_or(0).abs_diff(duration));
  if let Some(c)=candidate.filter(|c|duration==0||c["duration"].as_u64().is_some_and(|d|d.abs_diff(duration)<=15000)){
   let krc_request=http.get("https://lyrics.kugou.com/download").query(&[("ver","1"),("client","pc"),("id",c["id"].as_str().unwrap_or("")),("accesskey",c["accesskey"].as_str().unwrap_or("")),("fmt","krc"),("charset","utf8")]).send().await;
   if let Ok(response)=krc_request{if response.status().is_success(){if let Ok(content)=response.json::<Value>().await{if let Some(encoded)=content["content"].as_str(){if let Ok(text)=crate::krc::decode(encoded){if text.contains('<')&&text.contains('>'){return Ok(json!({"text":text,"format":"krc","provider":"酷狗逐字歌词","title":c["song"],"artist":c["singer"],"durationMs":c["duration"]}))}}}}}}
   let content:Value=http.get("https://lyrics.kugou.com/download").query(&[("ver","1"),("client","pc"),("id",c["id"].as_str().unwrap_or("")),("accesskey",c["accesskey"].as_str().unwrap_or("")),("fmt","lrc"),("charset","utf8")]).send().await.map_err(|_|"歌词下载失败")?.json().await.map_err(|_|"歌词下载返回异常")?;
   if let Some(encoded)=content["content"].as_str().filter(|s|s.len()<1_000_000){if let Ok(bytes)=STANDARD.decode(encoded){if let Ok(text)=String::from_utf8(bytes){if !text.trim().is_empty(){return Ok(json!({"text":text,"provider":"酷狗歌词","title":c["song"],"artist":c["singer"],"durationMs":c["duration"]}))}}}}
  }
 }
 let mut query=vec![("track_name",title.to_string()),("artist_name",artist.to_string())];if duration>0{query.push(("duration",(duration/1000).to_string()))}
 let response=http.get("https://lrclib.net/api/get").query(&query).send().await.map_err(|_|"备用歌词服务连接失败")?;
 if response.status()==reqwest::StatusCode::NOT_FOUND{return Err("没有匹配到这首歌的歌词，可调整歌名、歌手重试，或导入 LRC 歌词文件".into())}
 let data:Value=response.error_for_status().map_err(|_|"备用歌词服务暂不可用")?.json().await.map_err(|_|"备用歌词数据格式异常")?;
 if data["instrumental"]==true{return Ok(json!({"text":"","provider":"LRCLIB","instrumental":true}))}
 let text=data["syncedLyrics"].as_str().filter(|s|!s.trim().is_empty()).or_else(||data["plainLyrics"].as_str()).unwrap_or("");
 if text.is_empty(){return Err("没有可用歌词，可导入 LRC 歌词文件".into())}
 Ok(json!({"text":text,"provider":"LRCLIB","title":data["trackName"],"artist":data["artistName"]}))
}

#[tauri::command]
pub async fn get_lyrics(title:String,artist:String,source_id:Option<String>,duration_ms:Option<u64>,state:tauri::State<'_,AppState>)->Result<Value>{
 if title.trim().is_empty()||title.len()>500||artist.len()>500{return Err("歌名或歌手格式不正确".into())}
 let key=serde_json::to_string(&json!(["word-timed-v2",title,artist,source_id,duration_ms])).map_err(|e|e.to_string())?;
 {let db=state.db.lock().map_err(|e|e.to_string())?;db.execute("CREATE TABLE IF NOT EXISTS lyrics_cache(key TEXT PRIMARY KEY,payload TEXT NOT NULL)",[]).map_err(|e|e.to_string())?;if let Ok(raw)=db.query_row::<String,_,_>("SELECT payload FROM lyrics_cache WHERE key=?1",[&key],|row|row.get(0)){if let Ok(value)=serde_json::from_str(&raw){return Ok(value)}}}
 let http=Client::builder().timeout(Duration::from_secs(15)).user_agent("NekoPlayer/0.1.0 (desktop lyric lookup)").build().map_err(|e|e.to_string())?;
 let result=fetch(&http,&title,&artist,source_id.as_deref(),duration_ms.unwrap_or(0)).await?;
 let raw=serde_json::to_string(&result).map_err(|e|e.to_string())?;state.db.lock().map_err(|e|e.to_string())?.execute("INSERT OR REPLACE INTO lyrics_cache(key,payload) VALUES(?1,?2)",[key,raw]).map_err(|e|e.to_string())?;Ok(result)
}
