export type PlaybackMode='sequential'|'list'|'single'|'shuffle'
export function nextIndex(length:number,current:number,mode:PlaybackMode,delta=1,automatic=false,random=Math.random):number{
 if(!length)return -1
 if(automatic&&mode==='single')return Math.max(0,current)
 if(mode==='shuffle'&&length>1){const offset=1+Math.floor(random()*(length-1));return (Math.max(0,current)+offset)%length}
 const next=current+delta
 if(automatic&&mode==='sequential'&&next>=length)return -1
 return (next+length)%length
}
export function insertNext<T extends {id:string}>(items:T[],currentId:string|undefined,track:T):T[]{
 const existing=items.find(t=>t.id===currentId)
 const rest=items.filter(t=>t.id!==track.id)
 const index=existing?rest.findIndex(t=>t.id===existing.id):-1
 // Selecting the currently playing song queues another play without losing its position.
 if(track.id===currentId){const i=items.findIndex(t=>t.id===currentId);const copy=[...items];copy.splice(i+1,0,track);return copy}
 rest.splice(index+1,0,track);return rest
}
