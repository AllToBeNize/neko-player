use crate::{credential, err, AppState, Result};
use reqwest::{cookie::Jar, Client};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "bilibili-login";

#[tauri::command]
pub async fn open_web_login(app: tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        window.show().map_err(err)?;
        return window.set_focus().map_err(err);
    }
    let data = app.path().app_cache_dir().map_err(err)?.join("bilibili-login");
    WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::External(
        "https://passport.bilibili.com/login".parse().map_err(err)?,
    ))
    .title("B 站官方登录 · 可选择密码或短信登录")
    .inner_size(1080.0, 760.0)
    .min_inner_size(850.0, 650.0)
    .incognito(true)
    .data_directory(data)
    .on_navigation(|url| {
        url.scheme() == "https" && url.host_str().is_some_and(|host| {
            host == "bilibili.com" || host.ends_with(".bilibili.com")
        })
    })
    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
    .build().map_err(|_| "无法打开官方登录窗口，请关闭旧窗口后重试".to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_web_login(app: tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) { window.close().map_err(err)?; }
    Ok(())
}

#[tauri::command]
pub async fn finish_web_login(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<Value> {
    let window = app.get_webview_window(LABEL).ok_or("登录窗口已关闭，请重新打开并完成登录")?;
    // Async command: WebView2 cookie retrieval must not run on the UI thread.
    let cookies = window.cookies_for_url("https://www.bilibili.com/".parse().map_err(err)?)
        .map_err(|_| "无法读取网页登录状态，请重试".to_string())?;
    let allowed = ["SESSDATA", "bili_jct", "DedeUserID", "DedeUserID__ckMd5", "sid", "buvid3", "buvid4", "b_nut"];
    let entries: Vec<String> = cookies.iter().filter(|c| allowed.contains(&c.name()))
        .filter(|c| c.domain().is_none_or(|d| d.trim_start_matches('.') == "bilibili.com"))
        .map(ToString::to_string).collect();
    if !cookies.iter().any(|c| c.name() == "SESSDATA" && !c.value().is_empty()) {
        return Err("还未检测到登录，请先在官方窗口完成短信或密码登录，再点击连接账号".into());
    }
    // Validate separately: an incomplete or invalid login must not replace the current account.
    let jar = Arc::new(Jar::default());
    let origin = "https://www.bilibili.com/".parse().map_err(err)?;
    for cookie in &entries { jar.add_cookie_str(cookie, &origin); }
    let client = Client::builder().cookie_provider(jar)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/130.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(20)).build().map_err(err)?;
    let response: Value = client.get("https://api.bilibili.com/x/web-interface/nav")
        .header("Referer", "https://www.bilibili.com/").send().await
        .map_err(|_| "验证登录失败，请检查网络后重试")?.json().await
        .map_err(|_| "B 站返回的账号信息无效")?;
    if response["code"] != 0 || response["data"]["isLogin"] != true {
        return Err("网页登录状态尚未生效，请完成验证后重试".into());
    }
    if app.get_webview_window(LABEL).is_none() { return Err("登录已取消".into()); }
    credential()?.set_password(&serde_json::to_string(&entries).map_err(err)?)
        .map_err(|_| "无法保存 Windows 凭据，请重试")?;
    for cookie in &entries { state.jar.add_cookie_str(cookie, &origin); }
    let _ = window.close();
    let data = &response["data"];
    Ok(json!({"name":data["uname"],"face":data["face"]}))
}
