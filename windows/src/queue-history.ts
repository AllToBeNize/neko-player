import type { PlaybackMode } from './playback'
export type QueueEntry<T>={entryId:string;track:T;inserted:boolean;addedAt:number}
export type QueueSession<T>={id:string;name:string;entries:QueueEntry<T>[];cursorId:string;positions:Record<string,number>;pending:string[];mode:PlaybackMode;updatedAt:number;recovered?:boolean;cleared?:boolean}
export function makeSession<T extends {id:string}>(name:string,tracks:T[],selectedId:string,mode:PlaybackMode):QueueSession<T>{
 const entries=tracks.map(track=>({entryId:crypto.randomUUID(),track:{...track},inserted:false,addedAt:Date.now()}));return{id:crypto.randomUUID(),name,entries,cursorId:entries.find(e=>e.track.id===selectedId)?.entryId||entries[0]?.entryId||'',positions:{},pending:[],mode,updatedAt:Date.now()}
}
export function insertEntry<T extends {id:string}>(session:QueueSession<T>,track:T,nextOnly=false):QueueEntry<T>{
 if(!nextOnly){const existing=session.entries.find(e=>e.track.id===track.id);if(existing)return existing}
 session.cleared=false;const entry={entryId:crypto.randomUUID(),track:{...track},inserted:true,addedAt:Date.now()};const at=session.entries.findIndex(e=>e.entryId===session.cursorId);session.entries.splice(at+1,0,entry);if(nextOnly)session.pending.unshift(entry.entryId);session.updatedAt=Date.now();return entry
}
export function resumePosition(position:number|undefined,duration:number){const value=position??0;return Number.isFinite(value)&&value>=0&&(!duration||value<duration-1)?value:0}
export function compactSessions<T extends {id:string;source:string;url?:string;candidates?:unknown;customLyrics?:string;customTranslation?:string}>(sessions:QueueSession<T>[]):QueueSession<T>[]{
 return sessions.slice(0,10).map(s=>{const entries=s.entries.map(e=>{const {candidates,customLyrics,customTranslation,url,...rest}=e.track;return {...e,track:rest as T}});const ids=new Set(entries.map(e=>e.entryId));return {...s,entries,cursorId:ids.has(s.cursorId)?s.cursorId:entries[0]?.entryId||'',positions:Object.fromEntries(Object.entries(s.positions).filter(([id,value])=>ids.has(id)&&Number.isFinite(value)&&value>=0)),pending:s.pending.filter(id=>ids.has(id))}}).filter(s=>s.entries.length>0||s.cleared===true)
}

// Move existing entries by identity so duplicate recordings keep their own progress.
export function moveQueueEntry<T>(session:QueueSession<T>,id:string,targetId:string,after=false){
 if(id===targetId)return;const from=session.entries.findIndex(e=>e.entryId===id);if(from<0||!session.entries.some(e=>e.entryId===targetId))return
 const [entry]=session.entries.splice(from,1);const target=session.entries.findIndex(e=>e.entryId===targetId);session.entries.splice(target+(after?1:0),0,entry)
 const pending=new Set(session.pending);session.pending=session.entries.filter(e=>pending.has(e.entryId)).map(e=>e.entryId);session.updatedAt=Date.now()
}
export function queueEntryNext<T>(session:QueueSession<T>,id:string){
 if(id===session.cursorId)return;const entry=session.entries.find(e=>e.entryId===id);if(!entry)return
 moveQueueEntry(session,id,session.cursorId,true);entry.inserted=true;session.pending=[id,...session.pending.filter(p=>p!==id)];session.updatedAt=Date.now()
}
export function removeQueueEntry<T>(session:QueueSession<T>,id:string){
 const index=session.entries.findIndex(e=>e.entryId===id);if(index<0)return
 const active=session.cursorId===id;session.entries.splice(index,1);delete session.positions[id];session.pending=session.pending.filter(p=>p!==id)
 if(active)session.cursorId=session.pending.find(p=>session.entries.some(e=>e.entryId===p))||session.entries[Math.min(index,session.entries.length-1)]?.entryId||''
 if(active)session.pending=session.pending.filter(p=>p!==session.cursorId)
 session.cleared=session.entries.length===0;session.updatedAt=Date.now()
}
