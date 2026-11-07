export type TrackAudioSetting={balanceEnabled?:boolean;gainDb?:number}
export type TrackAudioSettings=Record<string,TrackAudioSetting>
type AudioTrack={id:string;source:string;sourceId?:string;bvid?:string;title:string;localFileSize?:number}
export function trackAudioKey(track:AudioTrack){
 if(track.source==='kugou'&&track.sourceId)return `kugou:${track.sourceId.toLowerCase()}`
 if(track.source==='local'&&track.localFileSize!==undefined)return `local:${JSON.stringify([track.title,track.localFileSize])}`
 if(track.source==='bilibili'&&track.bvid)return `bilibili:${track.bvid}`
 return `track:${track.id}`
}
export function clampTrackGain(value:number){return Number.isFinite(value)?Math.round(Math.max(-12,Math.min(12,value))*10)/10:0}
export function readTrackAudioSettings(value:unknown):TrackAudioSettings{
 const result:TrackAudioSettings={};if(!value||typeof value!=='object'||Array.isArray(value))return result
 for(const [key,item] of Object.entries(value)){if(!item||typeof item!=='object'||Array.isArray(item))continue;const setting=item as TrackAudioSetting,clean:TrackAudioSetting={};if(typeof setting.balanceEnabled==='boolean')clean.balanceEnabled=setting.balanceEnabled;if(typeof setting.gainDb==='number'&&Number.isFinite(setting.gainDb))clean.gainDb=clampTrackGain(setting.gainDb);if(Object.keys(clean).length)Object.defineProperty(result,key,{value:clean,enumerable:true,writable:true,configurable:true})}
 return result
}
