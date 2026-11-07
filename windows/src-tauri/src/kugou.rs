use reqwest::Client;
use serde_json::{json, Value};
use std::{collections::HashSet, time::Duration};

type Result<T> = std::result::Result<T, String>;

fn safe_url(mut url: url::Url) -> Result<url::Url> {
    if !matches!(url.scheme(), "http" | "https")
        || !url.host_str().is_some_and(|h| h == "kugou.com" || h.ends_with(".kugou.com"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err("请提供酷狗官方分享链接".into());
    }
    // Short links may advertise HTTP; never send share parameters over HTTP.
    url.set_scheme("https").map_err(|_| "无法转换为 HTTPS 链接")?;
    Ok(url)
}

async fn fetch_json(client: &Client, url: url::Url) -> Result<Value> {
    let response = client.get(url).send().await
        .map_err(|_| "酷狗歌单读取失败，请检查网络后重试")?
        .error_for_status().map_err(|_| "酷狗接口暂不可用，请稍后重试")?;
    if response.content_length().is_some_and(|n| n > 8_000_000) {
        return Err("酷狗返回数据过大，已停止导入".into());
    }
    let bytes = response.bytes().await.map_err(|_| "酷狗返回数据不完整")?;
    if bytes.len() > 8_000_000 { return Err("酷狗返回数据过大".into()); }
    serde_json::from_slice(&bytes).map_err(|_| "酷狗未返回有效歌单数据".into())
}

fn track(item: &Value, position: usize) -> Result<Value> {
    let raw = item["name"].as_str().or_else(|| item["filename"].as_str())
        .filter(|s| !s.trim().is_empty()).ok_or("歌单包含缺少歌名的条目，已停止导入")?;
    let (artist, title) = raw.split_once(" - ").unwrap_or(("", raw));
    let hash = item["hash"].as_str().unwrap_or("");
    let ms = item["timelen"].as_u64().unwrap_or(0);
    let cover = item["trans_param"]["union_cover"].as_str().map(|c| {
        c.replace("http://", "https://").replace("{size}", "240")
    });
    Ok(json!({
        "id": format!("kugou:{hash}:{position}"), "title": title.trim(),
        "artist": artist.trim(), "source": "kugou", "sourceId": hash,
        "duration": if ms > 0 { format!("{}:{:02}", ms / 60000, ms / 1000 % 60) } else { String::new() },
        "durationMs": ms, "cover": cover
    }))
}

pub async fn import(input: String) -> Result<Value> {
    let re = regex::Regex::new(r#"https?://[^\s<>「」]+"#).map_err(|e| e.to_string())?;
    let raw = re.find(&input).ok_or("请粘贴包含链接的酷狗分享文字")?.as_str()
        .trim_end_matches([')', ']', '，', '。', '】']);
    let mut current = safe_url(url::Url::parse(raw).map_err(|_| "分享链接格式错误")?)?;
    let client = Client::builder().redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/130.0.0.0 Mobile Safari/537.36")
        .build().map_err(|e| e.to_string())?;
    for hop in 0..=5 {
        // A resolved zlist URL already contains the signed playlist identity.
        if current.query_pairs().any(|(k, _)| k == "global_collection_id") { break; }
        let response = client.get(current.clone()).send().await.map_err(|_| "无法展开酷狗分享链接")?;
        if !response.status().is_redirection() {
            response.error_for_status().map_err(|_| "酷狗分享页面暂不可用")?;
            break;
        }
        if hop == 5 { return Err("酷狗分享链接跳转次数过多".into()); }
        let location = response.headers().get("location").and_then(|v| v.to_str().ok())
            .ok_or("酷狗分享跳转地址为空")?;
        current = safe_url(current.join(location).map_err(|_| "酷狗跳转地址格式错误")?)?;
    }
    let params: Vec<(String, String)> = current.query_pairs().filter(|(k, _)| {
        ["_t", "uid", "sign", "global_collection_id", "listid", "type", "token", "share_type", "chain", "from"].contains(&k.as_ref())
    }).map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    if params.iter().any(|(k, _)| k == "global_collection_id") {
        let mut out = Vec::new();
        let mut expected = None;
        let mut version = None;
        let mut name = String::from("酷狗导入歌单");
        let mut signatures = HashSet::new();
        for page in 1..=100 {
            let mut api = url::Url::parse("https://m3ws.kugou.com/zlist/list").unwrap();
            api.query_pairs_mut().extend_pairs(&params).append_pair("page", &page.to_string()).append_pair("pagesize", "100");
            let v = fetch_json(&client, api).await?;
            if v["status"].as_i64() != Some(1) || v["errcode"].as_i64().unwrap_or(0) != 0 {
                return Err(format!("酷狗歌单暂不可读取：{}", v["error"].as_str().filter(|s| !s.is_empty()).unwrap_or("链接失效或没有访问权限")));
            }
            let count = v["list"]["count"].as_u64().ok_or("酷狗未返回歌单总数，无法确认完整性")? as usize;
            let info = if v["info"].is_array() { &v["info"][0] } else { &v["info"] };
            let list_version = info.get("list_ver").ok_or("酷狗未返回歌单版本，无法确认完整性")?;
            if count > 10000 { return Err("本版单次最多导入 10000 首歌曲".into()); }
            if expected.is_some_and(|n| n != count) || version.as_ref().is_some_and(|n| n != list_version) {
                return Err("歌单在读取过程中发生变化，请重新导入；没有保存不完整结果".into());
            }
            expected = Some(count);
            version = Some(list_version.clone());
            if page == 1 { name = info["name"].as_str().unwrap_or("酷狗导入歌单").to_string(); }
            let items = v["list"]["info"].as_array().ok_or("酷狗返回的歌曲列表无效")?;
            if items.is_empty() && out.len() < count { return Err("酷狗分页提前结束，未保存不完整歌单".into()); }
            // Detect a server ignoring the requested page, while preserving legitimate repeated songs.
            if !items.is_empty() && !signatures.insert(serde_json::to_string(items).map_err(|e| e.to_string())?) {
                return Err("酷狗返回重复分页，已停止导入".into());
            }
            for item in items { out.push(track(item, out.len())?); }
            if out.len() == count { return Ok(json!({"name":name,"tracks":out,"total":count})); }
            if out.len() > count { return Err("酷狗返回数量与歌单总数不一致，已停止导入".into()); }
            tokio::time::sleep(Duration::from_millis(120)).await;
        }
        return Err("分页超过安全上限，未保存不完整歌单".into());
    }
    Err("当前链接不是已支持的酷狗分享歌单格式，请使用酷狗的歌单分享链接或歌曲清单导入".into())
}
