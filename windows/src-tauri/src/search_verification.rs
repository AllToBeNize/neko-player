use crate::{credential,err,signed_search_http,AppState,Result};
use reqwest::{Client,cookie::Jar};
use std::{sync::Arc,time::Duration};
use tauri::{Manager,WebviewUrl,WebviewWindowBuilder};
const LABEL:&str="bilibili-verification";
const UA:&str="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/130.0.0.0 Safari/537.36";
#[tauri::command]
pub async fn open_search_verification(app:tauri::AppHandle,keyword:String)->Result<()> {
    if let Some(w)=app.get_webview_window(LABEL){w.show().map_err(err)?;return w.set_focus().map_err(err)}
    let window=WebviewWindowBuilder::new(&app,LABEL,WebviewUrl::External("https://www.bilibili.com/".parse().map_err(err)?))
        .title("B 站官方搜索 · 请按页面提示完成安全验证")
        .inner_size(1100.0,780.0).incognito(true)
        .user_agent(UA)
        .data_directory(app.path().app_cache_dir().map_err(err)?.join("bilibili-verification"))
        .on_navigation(|u|u.scheme()=="https"&&u.host_str().is_some_and(|h|h=="bilibili.com"||h.ends_with(".bilibili.com")))
        .on_new_window(|_,_|tauri::webview::NewWindowResponse::Deny)
        .build().map_err(|_|"无法打开 B 站官方验证窗口")?;
    let setup=(||->Result<()>{
        let raw=credential()?.get_password().map_err(|_|"请先登录 B 站账号")?;
        let saved:Vec<String>=serde_json::from_str(&raw).map_err(|_|"本机登录凭据格式异常")?;
        for value in saved {
            let cookie=tauri::webview::Cookie::parse(value).map_err(|_|"无法读取登录会话")?;
            window.set_cookie(cookie).map_err(|_|"无法同步官方页面的本机登录会话")?;
        }
        let mut url=url::Url::parse("https://search.bilibili.com/all").map_err(err)?;
        url.query_pairs_mut().append_pair("keyword",&keyword);
        window.navigate(url).map_err(err)
    })();
    if setup.is_err(){let _=window.close();}setup
}
#[tauri::command]
pub async fn finish_search_verification(app:tauri::AppHandle,state:tauri::State<'_,AppState>,keyword:String)->Result<()> {
    let window=app.get_webview_window(LABEL).ok_or("官方验证窗口已关闭，请重新打开")?;
    let cookies=window.cookies_for_url("https://www.bilibili.com/".parse().map_err(err)?).map_err(|_|"无法读取官方页面会话")?;
    let allowed=["SESSDATA","bili_jct","DedeUserID","DedeUserID__ckMd5","sid","buvid3","buvid4","buvid_fp","b_nut","bili_ticket","bili_ticket_expires"];
    let entries:Vec<String>=cookies.into_iter().filter(|c|allowed.contains(&c.name())&&c.domain().is_some_and(|d|d.trim_start_matches('.')=="bilibili.com"))
        .map(|c|format!("{}={}; Domain=.bilibili.com; Path=/; Secure",c.name(),c.value())).collect();
    if !entries.iter().any(|c|c.starts_with("SESSDATA=")){return Err("请在官方窗口中登录原 B 站账号并完成安全验证".into())}
    let jar=Arc::new(Jar::default());let origin="https://www.bilibili.com/".parse().map_err(err)?;
    for c in &entries{jar.add_cookie_str(c,&origin)}
    let http=Client::builder().cookie_provider(jar).user_agent(UA).timeout(Duration::from_secs(20)).default_headers({let mut h=reqwest::header::HeaderMap::new();h.insert("Referer",reqwest::header::HeaderValue::from_static("https://www.bilibili.com/"));h}).build().map_err(err)?;
    // Login success alone is insufficient: verify the same search API that was challenged.
    signed_search_http(&http,keyword).await?;
    if app.get_webview_window(LABEL).is_none(){return Err("验证已取消".into())}
    credential()?.set_password(&serde_json::to_string(&entries).map_err(err)?).map_err(|_|"搜索已恢复，但无法保存验证后的系统凭据")?;
    for c in &entries{state.jar.add_cookie_str(c,&origin)}
    let _=window.close();Ok(())
}
