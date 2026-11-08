import type { TrackVersion } from './track-versions'
import { artistAliases,titleAliases } from './music-names'
export { sourceSearchKeyword } from './music-names'
export const MATCH_VERSION=5
export type MusicItem = {id:string; title:string; artist:string; duration?:string; durationMs?:number; bvid?:string; cover?:string; source:string;version?:TrackVersion}
export type Candidate = MusicItem & {score:number; reason:string;artistScore:number;titleScore:number;durationScore:number;officialHint?:boolean}

function decodeEntities(value:string){return value.replace(/&(?:amp|quot|apos|lt|gt|#39|#34);/gi,entity=>({'&amp;':'&','&quot;':'"','&apos;':"'",'&lt;':'<','&gt;':'>','&#39;':"'",'&#34;':'"'}[entity.toLowerCase()]||entity)).replace(/&#(x[\da-f]+|\d+);/gi,(raw,n)=>{const code=n[0].toLowerCase()==='x'?parseInt(n.slice(1),16):Number(n);return code>0&&code<=0x10ffff?String.fromCodePoint(code):raw})}
const clean=(value:string)=>decodeEntities(value.replace(/<[^>]+>/g,'')).toLowerCase().normalize('NFKC')
 .replace(/official\s*(music\s*)?(video|audio)?|music\s*video|lyrics?|完整版|官方|音乐视频|高音质|无损|音质|字幕版|中日字幕|中英字幕|动态歌词|纯享版|\bmv\b|\bpv\b|\bhd\b|\b4k\b|\b1080p\b/gi,'')
 .replace(/[^\p{L}\p{N}]/gu,'')
function overlap(a:string,b:string){if(a===b)return 1;if(!a||!b)return 0;const grams=(s:string)=>new Set(Array.from({length:Math.max(0,s.length-1)},(_,i)=>s.slice(i,i+2)));const aa=grams(a),bb=grams(b);if(!aa.size||!bb.size)return 0;let n=0;for(const x of aa)if(bb.has(x))n++;return 2*n/(aa.size+bb.size)}
function seconds(t:MusicItem){if(t.durationMs&&t.durationMs>0)return t.durationMs/1000;const parts=t.duration?.split(':').map(Number);if(!parts?.length||parts.some(n=>!Number.isFinite(n)))return 0;return parts.reduce((n,p)=>n*60+p,0)}
const versions:{pattern:RegExp;label:string;penalty:number;generic?:boolean}[]=[
 {pattern:/翻唱|\bcover\b|歌ってみた|歌ってみました/i,label:'翻唱版本',penalty:.30},
 {pattern:/现场|演唱会|\blive\b|concert/i,label:'现场版本',penalty:.23},
 {pattern:/伴奏|卡拉ok|纯音乐|instrumental|karaoke|off\s*vocal/i,label:'伴奏版本',penalty:.30},
 ...[/钢琴|鋼琴|\bpiano\b|ピアノ/i,/吉他|指弹|指彈|\bguitar\b|ギター/i,/小提琴|\bviolin\b|バイオリン|ヴァイオリン/i,/大提琴|\bcello\b/i,/萨克斯|薩克斯|\bsax(?:ophone)?\b/i,/口琴|\bharmonica\b/i,/古筝|古箏/i,/二胡/i,/琵琶/i,/扬琴|揚琴/i,/竖琴|豎琴|\bharp\b/i,/架子鼓|\bdrums?\b|叩いてみた/i,/八音盒|music\s*box/i,/手风琴|手風琴|\baccordion\b/i,/长笛|長笛|\bflute\b/i].map(pattern=>({pattern,label:'乐器改编',penalty:.32})),
 {pattern:/弾いてみた|演奏してみた/i,label:'乐器改编',penalty:.32,generic:true},
 {pattern:/混音|remix|加速|降调|升调|变速|slowed|sped\s*up/i,label:'混音或变速',penalty:.23},
 {pattern:/合集|串烧|串燒|精选集|精選集|全专辑|全專輯|\bmedley\b|\bcompilation\b|\d+\s*小时\s*(?:循环|连播)|\d+\s*hours?\s*(?:loop|mix)/i,label:'合集版本',penalty:.23}
]
export function versionSearchTerms(title:string){const instrument=versions.some(v=>v.label==='乐器改编'&&!v.generic&&v.pattern.test(title));return [...new Set(versions.filter(v=>!instrument||!v.generic&&v.label!=='翻唱版本').flatMap(v=>title.match(v.pattern)?.[0]||[]))]}
const official=/官方(?:MV|音源|音乐|音乐录影带|视频)?|\bofficial\b|オフィシャル|公式(?:MV|音源|ミュージック|映像|動画)?/i

export function rankCandidates(song:MusicItem, results:MusicItem[]):{ranked:Candidate[]; automatic:boolean}{
 const artist=clean(song.artist),knownDuration=seconds(song),artistNames=artistAliases(song.artist).map(clean).filter(name=>name.length>=2),titles=titleAliases(song.version?.baseTitle||song.title)
 const unique=new Map<string,MusicItem>();for(const item of results)if(item.bvid&&!unique.has(item.bvid))unique.set(item.bvid,item)
 const ranked:Candidate[]=Array.from(unique.values()).map(raw=>{
  const candidate={...raw,title:decodeEntities(raw.title)},candidateTitle=clean(candidate.title),uploader=clean(candidate.artist)
  const reduced=[...artistNames].sort((a,b)=>b.length-a.length).reduce((text,name)=>text.split(name).join(''),candidateTitle)
  let titleScore=0,aliasReason=''
  for(const variant of titles){const name=clean(variant.text);if(!name)continue;let similarity=overlap(name,reduced)
   if(reduced.includes(name)){const extra=reduced.length-name.length;similarity=Math.max(similarity,name.length>=4||extra===0?1:extra<=6?.82:.65)}
   // Loanwords and secondary bracket names are useful but weaker than the original name.
   const score=similarity*(variant.kind==='loanword'?.94:variant.kind==='bracket'?.92:1)
   if(score>titleScore){titleScore=score;aliasReason=variant.kind==='loanword'?'片假名/英文别名':variant.kind==='bracket'?'括号别名匹配':''}
  }
  const artistEvidence=artistNames.some(name=>candidateTitle.includes(name)||uploader.includes(name)||name.includes(uploader)&&uploader.length>=3)
  const artistScore=artistEvidence?1:0
  const candidateDuration=seconds(candidate),difference=knownDuration&&candidateDuration?Math.abs(knownDuration-candidateDuration):undefined
  const durationScore=difference===undefined?.5:difference<=4?1:difference<=10?.9:difference<=20?.65:difference<=35?.3:0
  const wantedVersion=song.version?song.version.referenceTitle+' '+song.version.name:song.title+' '+song.artist
  const mismatches=versions.filter(v=>v.pattern.test(candidate.title)&&!v.pattern.test(wantedVersion))
  const wantedInstrument=versions.some(v=>v.label==='乐器改编'&&!v.generic&&v.pattern.test(wantedVersion))
  const missingVersions=song.version?versions.filter(v=>(!wantedInstrument||!v.generic&&v.label!=='翻唱版本')&&v.pattern.test(wantedVersion)&&!v.pattern.test(candidate.title)):[]
  const penalty=[...mismatches,...missingVersions].reduce((max,v)=>Math.max(max,v.penalty),0)
  const longMismatch=difference!==undefined&&difference>Math.max(40,knownDuration*.22)
  // A declared official tag helps only when artist/title already agree and the version agrees.
  // It is a title hint, not proof of account certification.
  const officialBonus=official.test(candidate.title)&&!/非官方|非公式|unofficial|fan[ -]?made/i.test(candidate.title)&&artistScore>=.6&&titleScore>=.65&&!mismatches.length&&!missingVersions.length&&!longMismatch?.07:0
  let score=.55*artistScore+.35*titleScore+.1*durationScore+officialBonus-penalty
  if(longMismatch)score-=.22
  if(!artist)score-=.05
  score=Math.round(Math.max(0,Math.min(1,score))*100)
  const reason=[`歌手 ${Math.round(artistScore*100)}%`,`歌名 ${Math.round(titleScore*100)}%`,difference===undefined?'无可比时长':`时长差 ${Math.round(difference)} 秒`,aliasReason,officialBonus?'官方标题标记加分':'',...new Set(mismatches.map(v=>v.label+'降权')),...new Set(missingVersions.map(v=>'缺少目标'+v.label)),longMismatch?'时长差距过大':''].filter(Boolean).join(' · ')
  return {...candidate,score,reason,officialHint:officialBonus>0,artistScore:Math.round(artistScore*100),titleScore:Math.round(titleScore*100),durationScore:Math.round(durationScore*100)}
 }).sort((a,b)=>b.score-a.score||Number(b.officialHint)-Number(a.officialHint)||b.titleScore-a.titleScore)
 return {ranked:ranked.slice(0,8),automatic:ranked.length>0}
}
