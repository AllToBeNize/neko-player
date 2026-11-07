use serde_json::Value;
pub const VERIFICATION: &str = "BILIBILI_VERIFICATION_REQUIRED: B 站要求安全验证，已暂停搜索。请打开官方搜索页完成验证，再重新检查。";
pub fn data(value: Value) -> Result<Value,String> {
    if value["data"]["v_voucher"].as_str().is_some_and(|s|!s.is_empty())
        || matches!(value["code"].as_i64(),Some(-352)|Some(-412)) {
        return Err(VERIFICATION.into());
    }
    if value["code"]!=0 {return Err(format!("B 站搜索失败：{}（{}）",value["message"].as_str().unwrap_or("请求异常"),value["code"]))}
    if !value["data"]["result"].is_array() {return Err("B 站搜索响应缺少结果列表，未将其当作没有找到歌曲".into())}
    if value["data"]["result"].as_array().is_some_and(|a|a.is_empty())&&value["data"]["numResults"].as_u64().is_some_and(|n|n>0) {return Err("B 站搜索总数与结果列表不一致，请稍后重试".into())}
    Ok(value["data"].clone())
}
