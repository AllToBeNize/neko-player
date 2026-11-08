import { invoke } from '@tauri-apps/api/core'
import { rankCandidates,type MusicItem } from './matching'
import { sourceSearchQueries } from './music-names'
// Bounded adaptive retrieval: weak results expand the query, all results use the same identity scorer.
export async function searchSourceCandidates(song:MusicItem,options:{keyword?:string;active?:()=>boolean;progress?:(index:number,total:number,keyword:string)=>void}={}){
 const queries=options.keyword?[options.keyword]:sourceSearchQueries(song),pool=new Map<string,MusicItem>()
 for(let i=0;i<queries.length;i++){
  if(options.active&&!options.active())return []
  if(i){await new Promise(resolve=>setTimeout(resolve,4000));if(options.active&&!options.active())return []}
  options.progress?.(i+1,queries.length,queries[i])
  // Verification/network errors propagate immediately; no query switching to evade a challenge.
  const items=await invoke<MusicItem[]>('search',{keyword:queries[i]})
  if(options.active&&!options.active())return []
  for(const item of items)if(item.bvid&&!pool.has(item.bvid))pool.set(item.bvid,item)
  const best=rankCandidates(song,[...pool.values()]).ranked[0]
  if(best&&best.score>=85&&best.artistScore>=88&&best.titleScore>=88)break
 }
 return [...pool.values()]
}
