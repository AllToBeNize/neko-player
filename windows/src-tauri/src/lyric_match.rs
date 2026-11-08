use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
pub fn normalize(s:&str)->String{s.to_lowercase().chars().filter(|c|c.is_alphanumeric()).collect()}
pub fn title_names(title:&str,extra:&[String])->Vec<String>{let re=Regex::new(r"[（(]([^）)]+)[）)]").unwrap();let mut names=vec![title.to_string(),re.replace_all(title,"").trim().to_string()];for c in re.captures_iter(title){names.push(c[1].to_string())}names.extend(extra.iter().cloned());dedup(names,12)}
pub fn artist_names(artist:&str,extra:&[String])->Vec<String>{let re=Regex::new(r"[、,，&/／;；]|(?i:\s+feat\.?\s+|\s+ft\.?\s+)").unwrap();let mut names=vec![artist.to_string()];names.extend(re.split(artist).map(str::to_string));names.extend(extra.iter().cloned());dedup(names,12)}
pub fn dedup(names:Vec<String>,limit:usize)->Vec<String>{let mut seen=HashSet::new();names.into_iter().filter(|s|!normalize(s).is_empty()&&seen.insert(normalize(s))).take(limit).collect()}
fn similarity(a:&str,b:&str)->f64{let a=normalize(a);let b=normalize(b);if a==b{return 1.0}let aa:Vec<char>=a.chars().collect();let bb:Vec<char>=b.chars().collect();if aa.len()<4||bb.len()<4{return 0.0}let x:HashSet<_>=aa.windows(2).map(|c|(c[0],c[1])).collect();let y:HashSet<_>=bb.windows(2).map(|c|(c[0],c[1])).collect();2.0*x.intersection(&y).count() as f64/(x.len()+y.len()) as f64}
pub fn score(c:&Value,titles:&[String],artists:&[String],original:&str,duration:u64)->Option<f64>{
 let name=c["song"].as_str()?;let singer=c["singer"].as_str().unwrap_or("");let candidate_titles=title_names(name,&[]);
 let title=titles.iter().flat_map(|a|candidate_titles.iter().map(move|b|similarity(a,b))).fold(0.0,f64::max);if title<0.88{return None}
 let artist=artists.iter().flat_map(|a|artist_names(singer,&[]).into_iter().map(move|b|similarity(a,&b))).fold(0.0,f64::max);if !artists.is_empty()&&artist<0.88{return None}
 for group in [r"(?i)钢琴|鋼琴|piano|ピアノ",r"(?i)吉他|guitar|ギター",r"(?i)现场|演唱会|\blive\b|concert",r"(?i)翻唱|\bcover\b|歌ってみた",r"(?i)伴奏|instrumental|karaoke",r"(?i)remix|混音|加速|slowed"]{let re=Regex::new(group).unwrap();if re.is_match(original)!=re.is_match(name){return None}}
 let difference=if duration>0{let d=c["duration"].as_u64()?;let diff=d.abs_diff(duration);if diff>15000{return None}diff as f64}else{0.0};Some(title*60.0+artist*30.0+10.0*(1.0-difference/15000.0))
}
