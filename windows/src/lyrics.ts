export type LyricWord={time:number;duration:number;text:string}
export type LyricLine={time:number;text:string;duration?:number;words?:LyricWord[];translation?:string;romanization?:string}
export function parseLrc(raw:string):LyricLine[]{
 const offset=Number(raw.match(/\[offset:([+-]?\d+)\]/i)?.[1]||0)/1000
 const lines:LyricLine[]=[]
 for(const row of raw.split(/\r?\n/)){const stamps=Array.from(row.matchAll(/\[(\d+):(\d{1,2})(?:[.:](\d{1,3}))?\]/g));const text=row.replace(/\[[^\]]*\]/g,'').trim();if(!text)continue;for(const stamp of stamps){const seconds=Number(stamp[1])*60+Number(stamp[2])+Number(`0.${stamp[3]||'0'}`)-offset;lines.push({time:Math.max(0,seconds),text})}}
 lines.sort((a,b)=>a.time-b.time);const grouped:LyricLine[]=[];for(const line of lines){const last=grouped.at(-1);if(last&&Math.abs(last.time-line.time)<.001){if(!last.text.split('\n').includes(line.text))last.text+='\n'+line.text}else grouped.push({...line})}return grouped
}
export function lyricIndex(lines:LyricLine[],elapsed:number,offsetMs=0){const now=elapsed-offsetMs/1000;let low=0,high=lines.length-1,result=-1;while(low<=high){const mid=(low+high)>>1;if(lines[mid].time<=now){result=mid;low=mid+1}else high=mid-1}return result}

export function parseKrc(raw:string):LyricLine[]{
 const offset=Number(raw.match(/\[offset:([+-]?\d+)\]/i)?.[1]||0)/1000,lines:LyricLine[]=[]
 for(const row of raw.split(/\r?\n/)){const header=row.match(/^\s*\[(\d+),(\d+)\](.*)$/);if(!header)continue;const start=Number(header[1])/1000-offset,duration=Number(header[2])/1000;if(!Number.isFinite(start)||!Number.isFinite(duration)||start>172800)continue;const words:LyricWord[]=[];for(const word of header[3].matchAll(/<(\d+),(\d+),[^>]*>([^<]*)/g)){const time=start+Number(word[1])/1000,length=Number(word[2])/1000;if(Number.isFinite(time)&&Number.isFinite(length))words.push({time:Math.max(0,time),duration:Math.max(0,length),text:word[3]})}lines.push({time:Math.max(0,start),duration,words,text:words.length?words.map(w=>w.text).join('').trim():header[3].replace(/<[^>]*>/g,'').trim()})}
 const tag=raw.match(/\[language:([^\]]+)\]/)?.[1]
 if(tag){try{const data=JSON.parse(new TextDecoder().decode(Uint8Array.from(atob(tag),c=>c.charCodeAt(0))));const layers=[...(Array.isArray(data.content)?data.content:[]),...(Array.isArray(data.contentV2)?data.contentV2:[])];let translated=false;for(const layer of layers){if(!Array.isArray(layer.lyricContent))continue;const type=Number(layer.type);if(type!==0&&type!==1)continue;if(type===1&&translated)continue;layer.lyricContent.forEach((row:unknown,i:number)=>{if(!lines[i])return;const value=Array.isArray(row)?row.filter(v=>typeof v==='string').join(type===0?' ':''):typeof row==='string'?row:'';if(type===1)lines[i].translation=value;else lines[i].romanization=value});if(type===1)translated=true}}catch{/* Optional language metadata must not break word timing. */}}
 return lines.filter(l=>l.text).sort((a,b)=>a.time-b.time)
}
export function parseLyrics(raw:string):LyricLine[]{return /^\s*\[\d+,\d+\]/m.test(raw)?parseKrc(raw):parseLrc(raw)}
export function wordProgress(word:LyricWord,elapsed:number,offsetMs=0){const elapsedInWord=elapsed-offsetMs/1000-word.time;if(elapsedInWord<0)return 0;if(word.duration<=0)return 1;return Math.max(0,Math.min(1,elapsedInWord/word.duration))}
export function addTranslation(lines:LyricLine[],raw:string):LyricLine[]{if(!raw.trim())return lines;const timed=parseLrc(raw);if(timed.length){return lines.map(line=>{let closest:LyricLine|undefined;for(const candidate of timed){if(Math.abs(candidate.time-line.time)<=.75&&(!closest||Math.abs(candidate.time-line.time)<Math.abs(closest.time-line.time)))closest=candidate}return closest?{...line,translation:closest.text}:line})}const plain=raw.split(/\r?\n/).map(s=>s.trim()).filter(Boolean);return plain.length===lines.length?lines.map((line,i)=>({...line,translation:plain[i]})):lines}
