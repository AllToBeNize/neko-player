use crate::{AppState,Result};
use base64::{engine::general_purpose::STANDARD,Engine};
use reqwest::Client;
use serde_json::{json,Value};
use std::time::Duration;

async fn fetch(http:&Client,title:&str,artist:&str,hash:Option<&str>,duration:u64,title_aliases:&[String],artist_aliases:&[String])->Result<Value>{
 let titles=crate::lyric_match::title_names(title,title_aliases);let artists=crate::lyric_match::artist_names(artist,artist_aliases);
 let base=titles.iter().find(|s|!s.contains(['（','(','）',')'])).map(String::as_str).unwrap_or(title);
 let first=artists.get(1).or(artists.first()).map(String::as_str).unwrap_or("");
 let mut keywords=vec![format!("{first} {base}"),base.to_string()];if let Some(other)=artists.iter().find(|a|a.as_str()!=first&&a.as_str()!=artist){keywords.push(format!("{other} {base}"))}if let Some(alias)=titles.iter().find(|t|t.as_str()!=title&&t.as_str()!=base){keywords.push(format!("{first} {alias}"))}
 let keywords=crate::lyric_match::dedup(keywords,4);let mut candidates=Vec::new();let mut seen=std::collections::HashSet::new();
 for keyword in keywords{
  let mut request=http.get("https://lyrics.kugou.com/search").query(&[("ver","1"),("man","yes"),("client","pc"),("keyword",keyword.as_str()),("timelength",&duration.to_string())]);
  if let Some(hash)=hash.filter(|h|h.len()==32&&h.chars().all(|c|c.is_ascii_hexdigit())){request=request.query(&[("hash",hash)])}
  if let Ok(response)=request.send().await{if let Ok(response)=response.error_for_status(){if let Ok(data)=response.json::<Value>().await{if let Some(items)=data["candidates"].as_array(){for c in items{if c["id"].is_string()&&c["accesskey"].is_string()&&seen.insert(c["id"].to_string()){if let Some(score)=crate::lyric_match::score(c,&titles,&artists,title,duration){candidates.push((score,c.clone()))}}}}}}}
  if candidates.iter().any(|(score,_)|*score>=98.0){break}
  tokio::time::sleep(Duration::from_millis(250)).await;
 }
 candidates.sort_by(|a,b|b.0.total_cmp(&a.0));
 if let Some((_,c))=candidates.first(){
   let krc_request=http.get("https://lyrics.kugou.com/download").query(&[("ver","1"),("client","pc"),("id",c["id"].as_str().unwrap_or("")),("accesskey",c["accesskey"].as_str().unwrap_or("")),("fmt","krc"),("charset","utf8")]).send().await;
   if let Ok(response)=krc_request{if response.status().is_success(){if let Ok(content)=response.json::<Value>().await{if let Some(encoded)=content["content"].as_str(){if let Ok(text)=crate::krc::decode(encoded){if text.contains('<')&&text.contains('>'){return Ok(json!({"text":text,"format":"krc","provider":"酷狗逐字歌词","title":c["song"],"artist":c["singer"],"durationMs":c["duration"]}))}}}}}}
   let content:Value=http.get("https://lyrics.kugou.com/download").query(&[("ver","1"),("client","pc"),("id",c["id"].as_str().unwrap_or("")),("accesskey",c["accesskey"].as_str().unwrap_or("")),("fmt","lrc"),("charset","utf8")]).send().await.map_err(|_|"歌词下载失败")?.json().await.map_err(|_|"歌词下载返回异常")?;
   if let Some(encoded)=content["content"].as_str().filter(|s|s.len()<1_000_000){if let Ok(bytes)=STANDARD.decode(encoded){if let Ok(text)=String::from_utf8(bytes){if !text.trim().is_empty(){return Ok(json!({"text":text,"provider":"酷狗歌词","title":c["song"],"artist":c["singer"],"durationMs":c["duration"]}))}}}}
 }
 let mut query=vec![("track_name",base.to_string()),("artist_name",first.to_string())];if duration>0{query.push(("duration",(duration/1000).to_string()))}
 let response=http.get("https://lrclib.net/api/get").query(&query).send().await.map_err(|_|"备用歌词服务连接失败")?;
 if response.status()==reqwest::StatusCode::NOT_FOUND{return Err("已自动尝试原名、别名和歌手匹配，暂未找到版本与时长一致的歌词，可稍后重试或导入歌词".into())}
 let data:Value=response.error_for_status().map_err(|_|"备用歌词服务暂不可用")?.json().await.map_err(|_|"备用歌词数据格式异常")?;
 if data["instrumental"]==true{return Ok(json!({"text":"","provider":"LRCLIB","instrumental":true}))}
 let text=data["syncedLyrics"].as_str().filter(|s|!s.trim().is_empty()).or_else(||data["plainLyrics"].as_str()).unwrap_or("");
 if text.is_empty(){return Err("没有可用歌词，可导入 LRC 歌词文件".into())}
 Ok(json!({"text":text,"provider":"LRCLIB","title":data["trackName"],"artist":data["artistName"]}))
}

#[tauri::command]
pub async fn get_lyrics(title:String,artist:String,source_id:Option<String>,duration_ms:Option<u64>,title_aliases:Option<Vec<String>>,artist_aliases:Option<Vec<String>>,state:tauri::State<'_,AppState>)->Result<Value>{
 if title.trim().is_empty()||title.len()>500||artist.len()>500{return Err("歌名或歌手格式不正确".into())}
 let key=serde_json::to_string(&json!(["alias-scored-v3",title,artist,source_id,duration_ms])).map_err(|e|e.to_string())?;
 {let db=state.db.lock().map_err(|e|e.to_string())?;db.execute("CREATE TABLE IF NOT EXISTS lyrics_cache(key TEXT PRIMARY KEY,payload TEXT NOT NULL)",[]).map_err(|e|e.to_string())?;if let Ok(raw)=db.query_row::<String,_,_>("SELECT payload FROM lyrics_cache WHERE key=?1",[&key],|row|row.get(0)){if let Ok(value)=serde_json::from_str(&raw){return Ok(value)}}}
 let http=Client::builder().timeout(Duration::from_secs(15)).user_agent("NekoPlayer/0.1.0 (desktop lyric lookup)").build().map_err(|e|e.to_string())?;
 let result=fetch(&http,&title,&artist,source_id.as_deref(),duration_ms.unwrap_or(0),&title_aliases.unwrap_or_default(),&artist_aliases.unwrap_or_default()).await?;
 let raw=serde_json::to_string(&result).map_err(|e|e.to_string())?;state.db.lock().map_err(|e|e.to_string())?.execute("INSERT OR REPLACE INTO lyrics_cache(key,payload) VALUES(?1,?2)",[key,raw]).map_err(|e|e.to_string())?;Ok(result)
}
