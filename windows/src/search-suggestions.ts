export type Suggestion={text:string;kind:'history'|'library'|'music'|'remote';label:string}
export const musicRecommendations=['纯音乐','爵士钢琴','华语流行音乐','日系音乐','游戏原声音乐','轻音乐']
const normalize=(s:string)=>s.normalize('NFKC').toLowerCase().replace(/\s+/g,' ').trim()
export function buildSuggestions(query:string,history:string[],library:{title:string;artist:string}[],remote:string[]):Suggestion[]{
 const key=normalize(query),items:Suggestion[]=[],seen=new Set<string>();const add=(text:string,kind:Suggestion['kind'],label:string)=>{const clean=text.trim(),id=normalize(clean);if(!id||seen.has(id))return;seen.add(id);items.push({text:clean,kind,label})}
 if(/^https?:\/\//i.test(query)||/^bv[0-9a-z]{0,10}$/i.test(query.trim())){add(query,'music','直接查看音轨');return items}
 for(const word of history.filter(word=>!key||normalize(word).includes(key)).slice(0,key?3:5))add(word,'history','最近搜索')
 if(key){
  let count=0;for(const song of library){if(normalize(song.title).includes(key)||normalize(song.artist).includes(key)){add([song.artist,song.title].filter(Boolean).join(' '),'library','来自你的曲库');if(++count>=4)break}}
  const hasMusic=/歌|音乐|演唱|翻唱|伴奏|钢琴|原声|\blive\b|\bcover\b|\bmv\b|\bost\b|\bjazz\b|\bpiano\b/i
  if(!hasMusic.test(query)){add(`${query.trim()} 歌曲`,'music','音乐方向');add(`${query.trim()} 纯音乐`,'music','音乐方向')}
  for(const word of [...remote].sort((a,b)=>Number(hasMusic.test(b))-Number(hasMusic.test(a))))add(word,'remote','B 站联想')
  if(!items.length)add(query,'music','搜索这个关键词')
 }else for(const word of musicRecommendations)add(word,'music','推荐搜索')
 return items.slice(0,10)
}
