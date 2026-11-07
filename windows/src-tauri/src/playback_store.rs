use crate::Result;
use rusqlite::{Connection,OptionalExtension,TransactionBehavior};
use serde_json::Value;

fn setup(db:&Connection)->Result<()>{db.execute_batch("CREATE TABLE IF NOT EXISTS playback_state(id INTEGER PRIMARY KEY,payload TEXT NOT NULL);CREATE TABLE IF NOT EXISTS playback_backups(id INTEGER PRIMARY KEY AUTOINCREMENT,payload TEXT NOT NULL);").map_err(|e|e.to_string())}
fn nonempty(value:&Value)->bool{value["sessions"].as_array().is_some_and(|v|!v.is_empty())}
pub fn load(db:&Connection)->Result<Value>{
 setup(db)?;let raw:Option<String>=db.query_row("SELECT payload FROM playback_state WHERE id=1",[],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
 let current:Value=match raw{Some(raw)=>serde_json::from_str(&raw).map_err(|_|"播放记录文件格式异常，已停止自动覆盖")?,None=>Value::Null};
 if nonempty(&current){return Ok(current)}
 let mut statement=db.prepare("SELECT payload FROM playback_backups ORDER BY id DESC LIMIT 5").map_err(|e|e.to_string())?;
 let rows=statement.query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;
 for raw in rows.flatten(){if let Ok(mut backup)=serde_json::from_str::<Value>(&raw){if nonempty(&backup){backup["revision"]=current["revision"].as_u64().unwrap_or(0).into();backup["recoveredBackup"]=true.into();return Ok(backup)}}}Ok(current)
}
pub fn save(db:&mut Connection,mut value:Value,expected:Option<u64>)->Result<u64>{
 setup(db)?;if !value["sessions"].is_array(){return Err("播放队列格式无效".into())}
 let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
 let old:Option<String>=transaction.query_row("SELECT payload FROM playback_state WHERE id=1",[],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
 let previous:Value=match &old{Some(raw)=>serde_json::from_str(raw).map_err(|_|"已有播放记录异常，已拒绝覆盖")?,None=>Value::Null};
 let revision=previous["revision"].as_u64().unwrap_or(0);
 if expected.is_some_and(|n|n!=revision)||(expected.is_none()&&revision>0){return Err("历史队列已被更新，已阻止旧状态覆盖；请重新打开播放器读取最新记录".into())}
 if nonempty(&previous)&&!nonempty(&value){return Err("已阻止空队列覆盖历史记录".into())}
 let next=revision+1;value["revision"]=next.into();let raw=serde_json::to_string(&value).map_err(|e|e.to_string())?;
 if raw.len()>20_000_000{return Err("历史队列数据过大，请删除不需要的历史队列".into())}
 if nonempty(&previous){transaction.execute("INSERT INTO playback_backups(payload) VALUES(?1)",[old.as_deref().unwrap_or("")]).map_err(|e|e.to_string())?;transaction.execute("DELETE FROM playback_backups WHERE id NOT IN (SELECT id FROM playback_backups ORDER BY id DESC LIMIT 5)",[]).map_err(|e|e.to_string())?;}
 transaction.execute("INSERT INTO playback_state(id,payload) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",[raw]).map_err(|e|e.to_string())?;transaction.commit().map_err(|e|e.to_string())?;Ok(next)
}
