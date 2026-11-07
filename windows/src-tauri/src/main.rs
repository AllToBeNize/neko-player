#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod kugou;
mod web_login;
mod search_response;
mod search_verification;
mod lyrics;
mod suggestions;
mod playback_store;
mod krc;
use rusqlite::OptionalExtension;
use std::{sync::{Arc, Mutex}, collections::{HashMap, BTreeMap}, time::Duration};
use serde_json::{Value, json};
use reqwest::{Client, cookie::Jar};
use tauri::Manager;
use axum::{Router, routing::get, extract::{State, Path}, http::{HeaderMap, StatusCode}, body::Body, response::Response};

struct AppState { http: Client, jar: Arc<Jar>, db: Mutex<rusqlite::Connection>, proxy: Proxy, port: u16 }
#[derive(Clone)]
struct Proxy { http: Client, streams: Arc<Mutex<HashMap<String,String>>> }
type Result<T> = std::result::Result<T,String>;
fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn credential() -> Result<keyring::Entry> { keyring::Entry::new("NekoPlayer", "bilibili").map_err(err) }
fn allowed_media(raw: &str) -> bool { url::Url::parse(raw).ok().map(|u| u.scheme()=="https" && u.host_str().map(|h| ["bilivideo.com","bilivideo.cn","biliapi.net"].iter().any(|d| h==*d || h.ends_with(&format!(".{d}")))).unwrap_or(false)).unwrap_or(false) }
async fn proxy_stream(State(p): State<Proxy>, Path(id): Path<String>, headers: HeaderMap) -> std::result::Result<Response,StatusCode> {
 let raw=p.streams.lock().map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.get(&id).cloned().ok_or(StatusCode::NOT_FOUND)?;
 let mut req=p.http.get(raw).header("Referer","https://www.bilibili.com/");
 if let Some(range)=headers.get("range") { req=req.header("Range",range); }
 let r=req.send().await.map_err(|_|StatusCode::BAD_GATEWAY)?;
 let mut res=Response::builder().status(r.status()).header("Access-Control-Allow-Origin","*").header("Access-Control-Expose-Headers","Content-Length, Content-Range, Accept-Ranges");
 for key in ["content-type","content-length","content-range","accept-ranges"] { if let Some(v)=r.headers().get(key) { res=res.header(key,v); } }
 res.body(Body::from_stream(r.bytes_stream())).map_err(|_|StatusCode::BAD_GATEWAY)
}
async fn bili(s: &AppState, path: &str, params: &[(&str,String)]) -> Result<Value> {
 let v:Value=s.http.get(format!("https://api.bilibili.com{path}")).query(params).send().await.map_err(err)?.error_for_status().map_err(err)?.json().await.map_err(err)?;
 if v["code"].as_i64().unwrap_or(-1)!=0 { return Err(format!("B 站：{}（{}）", v["message"].as_str().unwrap_or("请求失败"),v["code"])); } Ok(v["data"].clone())
}
#[tauri::command]
async fn account(s:tauri::State<'_,AppState>) -> Result<Value> { let d=bili(&s,"/x/web-interface/nav",&[]).await?; Ok(json!({"name":d["uname"],"face":d["face"]})) }
#[tauri::command]
async fn login_qr(s:tauri::State<'_,AppState>) -> Result<Value> {
 let v:Value=s.http.get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate").send().await.map_err(err)?.json().await.map_err(err)?;
 let d=&v["data"]; let raw=d["url"].as_str().ok_or("无法生成二维码，请重试")?;
 let svg=qrcode::QrCode::new(raw).map_err(err)?.render::<qrcode::render::svg::Color>().min_dimensions(220,220).build();
 Ok(json!({"key":d["qrcode_key"],"svg":svg}))
}
#[tauri::command]
async fn login_poll(key:String,s:tauri::State<'_,AppState>) -> Result<Value> {
 let r=s.http.get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll").query(&[("qrcode_key",key)]).send().await.map_err(err)?;
 let cookies:Vec<String>=r.headers().get_all("set-cookie").iter().filter_map(|v|v.to_str().ok()).map(str::to_string).collect();
 let v:Value=r.json().await.map_err(err)?; let d=&v["data"];
 if d["code"]==0 { credential()?.set_password(&serde_json::to_string(&cookies).map_err(err)?).map_err(|_|"登录成功，但无法保存系统凭据".to_string())?; }
 Ok(d.clone())
}
#[tauri::command]
fn logout(app:tauri::AppHandle,s:tauri::State<'_,AppState>) -> Result<()> {
 if let Some(window)=app.get_webview_window("bilibili-login") { let _=window.close(); }
 if let Some(window)=app.get_webview_window("bilibili-verification") { let _=window.close(); }
 // Expire the host and domain variants used by web authentication.
 for domain in [".bilibili.com","bilibili.com","passport.bilibili.com","api.bilibili.com"] { for name in ["SESSDATA","bili_jct","DedeUserID","DedeUserID__ckMd5","sid"] { s.jar.add_cookie_str(&format!("{name}=; Domain={domain}; Path=/; Max-Age=0"),&"https://passport.bilibili.com/".parse().unwrap()); } }
 if let Ok(e)=credential() { let _=e.delete_credential(); } Ok(())
}
async fn signed_search(s:&AppState, keyword:String) -> Result<Value> {
 signed_search_http(&s.http,keyword).await
}
async fn signed_search_http(http:&Client, keyword:String) -> Result<Value> {
 let nav:Value=http.get("https://api.bilibili.com/x/web-interface/nav").send().await.map_err(err)?.json().await.map_err(err)?;
 let key=|name:&str| -> Result<String> { Ok(nav["data"]["wbi_img"][name].as_str().ok_or("无法获取搜索签名")?.rsplit('/').next().unwrap_or("").split('.').next().unwrap_or("").to_string()) };
 let original=format!("{}{}",key("img_url")?,key("sub_url")?);
 let table=[46,47,18,2,53,8,23,32,15,50,10,31,58,3,45,35,27,43,5,49,33,9,42,19,29,28,14,39,12,38,41,13];
 let mix:String=table.iter().filter_map(|&i|original.chars().nth(i)).collect();
 let mut p=BTreeMap::new(); p.insert("keyword",keyword.chars().filter(|c|!"!'()*".contains(*c)).collect::<String>()); p.insert("search_type","video".into()); p.insert("page","1".into()); p.insert("wts",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(err)?.as_secs().to_string());
 let query=url::form_urlencoded::Serializer::new(String::new()).extend_pairs(&p).finish().replace('+',"%20");
 let hash=format!("{:x}",md5::compute(format!("{query}{mix}")));
 let response=http.get(format!("https://api.bilibili.com/x/web-interface/wbi/search/type?{query}&w_rid={hash}")).send().await.map_err(|_|"B 站搜索连接失败，请检查网络".to_string())?;
 if matches!(response.status().as_u16(),412|429) {return Err(search_response::VERIFICATION.into())}
 let v:Value=response.error_for_status().map_err(|_|"B 站搜索服务请求失败".to_string())?.json().await.map_err(|_|"B 站没有返回有效搜索数据".to_string())?;
 search_response::data(v)
}
#[tauri::command]
async fn search(keyword:String,s:tauri::State<'_,AppState>) -> Result<Value> {
 if keyword.trim().is_empty() { return Err("请输入歌名或歌手".into()); }
 if let Some(found)=regex::Regex::new(r"(?i)\bBV[0-9A-Za-z]{10}\b").map_err(err)?.find(&keyword) {
 let bvid=format!("BV{}",&found.as_str()[2..]);
 let d=bili(&s,"/x/web-interface/view",&[("bvid",bvid.clone())]).await?;
 let seconds=d["pages"][0]["duration"].as_u64().or_else(||d["duration"].as_u64()).unwrap_or(0);
 return Ok(json!([{"id":bvid,"bvid":bvid,"title":d["title"],"artist":d["owner"]["name"],"cover":d["pic"],"duration":format!("{}:{:02}",seconds/60,seconds%60),"source":"bilibili"}]));
 }
 let d=signed_search(&s,keyword).await?;
 let strip=regex::Regex::new("<[^>]*>").map_err(err)?;
 let items:Vec<Value>=d["result"].as_array().unwrap_or(&Vec::new()).iter().filter(|x|x["bvid"].is_string()).map(|x|json!({"id":x["bvid"],"bvid":x["bvid"],"title":strip.replace_all(x["title"].as_str().unwrap_or(""),"").replace("&amp;","&"),"artist":x["author"],"cover":format!("https:{}",x["pic"].as_str().unwrap_or("").trim_start_matches("https:").trim_start_matches("http:")),"duration":x["duration"],"source":"bilibili"})).collect();
 if items.is_empty()&&d["result"].as_array().is_some_and(|a|!a.is_empty()){return Err("B 站搜索返回了无法识别的视频数据，请稍后重试".into())}
 Ok(json!(items))
}
fn recognition_entry() -> Result<keyring::Entry> { keyring::Entry::new("NekoPlayer", "audd-api-token").map_err(err) }
#[tauri::command]
fn recognition_config() -> bool { recognition_entry().and_then(|e|e.get_password().map_err(err)).map(|v|!v.is_empty()).unwrap_or(false) }
#[tauri::command]
fn save_recognition_token(token:String) -> Result<()> {
 let entry=recognition_entry()?;
 if token.trim().is_empty() {let _=entry.delete_credential();return Ok(())}
 if token.len()>512 {return Err("Token 格式不正确".into())}
 entry.set_password(token.trim()).map_err(|_|"无法保存系统凭据，请检查 Windows 凭据管理器".into())
}
#[tauri::command]
async fn recognize_audio(bytes:Vec<u8>) -> Result<Value> {
 if bytes.len()<44 || bytes.len()>5_000_000 || !bytes.starts_with(b"RIFF") {return Err("请提供不超过 5 MB 的 WAV 音频片段".into())}
 let token=recognition_entry()?.get_password().map_err(|_|"请先配置 AudD API Token".to_string())?;
 let part=reqwest::multipart::Part::bytes(bytes).file_name("clip.wav").mime_str("audio/wav").map_err(err)?;
 let form=reqwest::multipart::Form::new().text("api_token",token).part("file",part);
 // A separate client prevents Bilibili cookies or headers being sent to the recognition service.
 let v:Value=Client::builder().timeout(Duration::from_secs(40)).redirect(reqwest::redirect::Policy::none()).build().map_err(err)?.post("https://api.audd.io/").multipart(form).send().await.map_err(|_|"识曲服务连接失败，请检查网络".to_string())?.error_for_status().map_err(|_|"识曲服务请求失败，请检查服务状态".to_string())?.json().await.map_err(|_|"识曲服务返回格式异常".to_string())?;
 if v["status"]!="success" {return Err(format!("识别失败：{}",v["error"]["error_message"].as_str().unwrap_or("请检查 Token、配额或稍后重试")))}
 if v["result"].is_null() {return Err("没有识别到歌曲，请换一段更清晰、包含原曲的音乐重试。暂不支持哼唱识曲。".into())}
 Ok(json!({"title":v["result"]["title"],"artist":v["result"]["artist"],"album":v["result"]["album"]}))
}
#[tauri::command]
async fn resolve(bvid:String,validate_only:Option<bool>,s:tauri::State<'_,AppState>) -> Result<Value> {
 if !regex::Regex::new(r"^BV[0-9A-Za-z]{10}$").map_err(err)?.is_match(&bvid) { return Err("无效的视频编号".into()); }
 let d=bili(&s,"/x/web-interface/view",&[("bvid",bvid.clone())]).await?;
 let cid=d["cid"].as_u64().ok_or("视频没有可用分 P")?;
 let p=bili(&s,"/x/player/playurl",&[("bvid",bvid),("cid",cid.to_string()),("fnval","4048".into()),("fnver","0".into()),("fourk","1".into())]).await?;
 let audio=p["dash"]["audio"].as_array().and_then(|a|a.iter().filter(|x|x["codecs"].as_str().unwrap_or("").starts_with("mp4a")).max_by_key(|x|x["bandwidth"].as_u64().unwrap_or(0))).ok_or("该视频未返回可用 AAC 音轨，可能需要登录或没有播放权限")?;
 let raw=audio["baseUrl"].as_str().or_else(||audio["base_url"].as_str()).ok_or("音频地址为空")?;
 if !allowed_media(raw) { return Err("平台返回了未支持的媒体域名".into()); }
 if validate_only==Some(true) { return Ok(json!({"available":true})); }
 let id=uuid::Uuid::new_v4().to_string();
 {let mut streams=s.proxy.streams.lock().map_err(err)?; if streams.len()>30 { streams.clear(); } streams.insert(id.clone(),raw.into());}
 Ok(json!({"url":format!("http://127.0.0.1:{}/stream/{id}",s.port),"title":d["title"],"cover":d["pic"],"duration":d["duration"]}))
}
#[tauri::command]
fn load_library(s:tauri::State<'_,AppState>) -> Result<Value> {
 let db=s.db.lock().map_err(err)?;
 let raw:Option<String>=db.query_row("SELECT payload FROM library WHERE id=1",[],|row|row.get(0)).optional().map_err(err)?;match raw{Some(raw)=>serde_json::from_str(&raw).map_err(err),None=>Ok(Value::Null)}
}
#[tauri::command]
fn save_library(value:Value,s:tauri::State<'_,AppState>) -> Result<()> {
 let raw=serde_json::to_string(&value).map_err(err)?; if raw.len()>10_000_000 {return Err("歌单数据过大".into())}
 s.db.lock().map_err(err)?.execute("INSERT INTO library(id,payload) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",[raw]).map_err(err)?; Ok(())
}
#[tauri::command]
fn load_queue_state(s:tauri::State<'_,AppState>)->Result<Value>{
 let db=s.db.lock().map_err(err)?;playback_store::load(&db)
}
#[tauri::command]
fn save_queue_state(value:Value,expected_revision:Option<u64>,s:tauri::State<'_,AppState>)->Result<u64>{
 let mut db=s.db.lock().map_err(err)?;playback_store::save(&mut db,value,expected_revision)
}
#[tauri::command]
async fn import_kugou(input:String) -> Result<Value> { kugou::import(input).await }
fn main() {
 tauri::Builder::default().setup(|app|{
 let jar=Arc::new(Jar::default());
 if let Ok(e)=credential(){if let Ok(raw)=e.get_password(){if let Ok(cookies)=serde_json::from_str::<Vec<String>>(&raw){for c in cookies {jar.add_cookie_str(&c,&"https://passport.bilibili.com/".parse().unwrap());}}}}
 let http=Client::builder().cookie_provider(jar.clone()).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/130.0.0.0 Safari/537.36").default_headers({ let mut h=reqwest::header::HeaderMap::new(); h.insert("Referer",reqwest::header::HeaderValue::from_static("https://www.bilibili.com/")); h }).timeout(Duration::from_secs(25)).build()?;
 let media=Client::builder().user_agent("Mozilla/5.0").redirect(reqwest::redirect::Policy::custom(|a|if allowed_media(a.url().as_str()) && a.previous().len()<5 {a.follow()}else{a.stop()})).build()?;
 let proxy=Proxy{http:media,streams:Arc::new(Mutex::new(HashMap::new()))};
 let listener=std::net::TcpListener::bind("127.0.0.1:0")?;let port=listener.local_addr()?.port();listener.set_nonblocking(true)?;
 let router=Router::new().route("/stream/{id}",get(proxy_stream)).with_state(proxy.clone());
 tauri::async_runtime::spawn(async move {if let Ok(l)=tokio::net::TcpListener::from_std(listener){let _=axum::serve(l,router).await;}});
 let dir=app.path().app_data_dir()?;std::fs::create_dir_all(&dir)?;let db=rusqlite::Connection::open(dir.join("library.sqlite"))?;db.execute("CREATE TABLE IF NOT EXISTS library(id INTEGER PRIMARY KEY,payload TEXT NOT NULL)",[])?;
 app.manage(AppState{http,jar,db:Mutex::new(db),proxy,port}); Ok(())
 }).invoke_handler(tauri::generate_handler![account,login_qr,login_poll,logout,search,resolve,load_library,save_library,import_kugou,recognition_config,save_recognition_token,recognize_audio,web_login::open_web_login,web_login::close_web_login,web_login::finish_web_login,search_verification::open_search_verification,search_verification::finish_search_verification,lyrics::get_lyrics,suggestions::search_suggestions,load_queue_state,save_queue_state,krc::decode_lyric_file]).run(tauri::generate_context!()).expect("无法启动 Neko Player");
}
