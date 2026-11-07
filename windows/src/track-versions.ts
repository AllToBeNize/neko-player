import { MATCH_VERSION,rankCandidates,versionSearchTerms,type MusicItem,type Candidate } from './matching'
export type TrackVersion={parentId:string;baseTitle:string;name:string;referenceTitle:string;searchTerms:string[]}
export function suggestVersionName(title:string){
 const labels:[RegExp,string][]=[[/钢琴|鋼琴|piano|ピアノ/i,'钢琴版'],[/吉他|指弹|指彈|guitar|ギター/i,'吉他版'],[/小提琴|violin|バイオリン|ヴァイオリン/i,'小提琴版'],[/大提琴|cello/i,'大提琴版'],[/翻唱|cover|歌ってみた/i,'翻唱版'],[/现场|演唱会|\blive\b|concert/i,'现场版'],[/伴奏|instrumental|karaoke|off\s*vocal/i,'伴奏版'],[/混音|remix/i,'混音版']]
 return labels.find(([pattern])=>pattern.test(title))?.[1]||'独立版本'
}
export function makeIndependentVersion(original:MusicItem,candidate:Candidate,name:string){
 const label=name.trim().slice(0,60);if(!label||!candidate.bvid)throw Error('请选择歌源并填写版本名称')
 const version:TrackVersion={parentId:original.version?.parentId||original.id,baseTitle:original.version?.baseTitle||original.title,name:label,referenceTitle:candidate.title,searchTerms:versionSearchTerms(candidate.title)}
 // Build explicitly: source hashes, old lyrics, URLs, offsets and audio settings belong to the original recording.
 const track={id:`version:${crypto.randomUUID()}`,title:`${version.baseTitle} · ${label}`,artist:original.artist||candidate.artist,source:'version',bvid:candidate.bvid,cover:candidate.cover||original.cover,duration:candidate.duration,durationMs:candidate.durationMs,version,matchStatus:'manual' as const,matchVersion:MATCH_VERSION}
 const selected=rankCandidates(track,[candidate]).ranked[0]!
 return {...track,candidates:[selected],matchScore:selected.score,matchReason:selected.reason}
}
