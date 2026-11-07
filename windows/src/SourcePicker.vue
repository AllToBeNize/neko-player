<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Search, Play, Check, LoaderCircle, Radio, ListPlus, ArrowLeft } from 'lucide-vue-next'
import { suggestVersionName } from './track-versions'
import { rankCandidates, sourceSearchKeyword, type MusicItem, type Candidate } from './matching'
const props=defineProps<{song:MusicItem&{candidates?:Candidate[]};destinations:{id:string;name:string}[];defaultDestination:string}>()
const emit=defineEmits<{choose:[candidate:Candidate,candidates:Candidate[]];preview:[candidate:Candidate];blocked:[message:string,keyword:string];verify:[];version:[candidate:Candidate,name:string,playlistId:string]}>()
const keyword=ref(sourceSearchKeyword(props.song)),candidates=ref<Candidate[]>([]),chosen=ref(''),busy=ref(false),message=ref('')
const versionCandidate=ref<Candidate|null>(null),versionName=ref(''),destination=ref(props.defaultDestination)
function beginVersion(candidate:Candidate){versionCandidate.value=candidate;versionName.value=suggestVersionName(candidate.title);destination.value=props.defaultDestination||props.destinations[0]?.id||''}
function saveVersion(){if(versionCandidate.value&&versionName.value.trim()&&destination.value)emit('version',versionCandidate.value,versionName.value.trim(),destination.value)}
let generation=0,alive=true
function update(items:MusicItem[]){candidates.value=rankCandidates(props.song,items).ranked;chosen.value=candidates.value.some(t=>t.bvid===props.song.bvid)?props.song.bvid!:candidates.value[0]?.bvid||''}
async function search(){const run=++generation;busy.value=true;message.value='';try{const items=await invoke<MusicItem[]>('search',{keyword:keyword.value});if(alive&&run===generation){update(items);if(!items.length)message.value='没有找到候选，试试调整歌手或歌名。'}}catch(e){if(alive&&run===generation){message.value=String(e);if(message.value.includes('BILIBILI_VERIFICATION_REQUIRED'))emit('blocked',message.value,keyword.value)}}finally{if(alive&&run===generation)busy.value=false}}
function confirm(){const item=candidates.value.find(t=>t.bvid===chosen.value);if(item)emit('choose',item,candidates.value)}
onMounted(()=>{if(props.song.candidates?.length)update(props.song.candidates);else void search()})
onUnmounted(()=>{alive=false;generation++})
</script>
<template>
 <template v-if="versionCandidate"><button class="text-btn" @click="versionCandidate=null"><ArrowLeft :size="15"/>返回候选歌源</button><span class="modal-symbol"><ListPlus :size="25"/></span><h2>另存为独立版本</h2><p class="muted">{{song.version?.baseTitle||song.title}}<br/>{{versionCandidate.title}}</p><form class="version-form" @submit.prevent="saveVersion"><label>版本名称<input v-model="versionName" class="field" aria-label="版本名称" maxlength="60" placeholder="例如：钢琴版、人声版、现场版" autofocus/></label><label>加入歌单<select v-model="destination" class="field" aria-label="版本目标歌单"><option v-for="p in destinations" :key="p.id" :value="p.id">{{p.name}}</option></select></label><p class="info-box">新版本会单独保存音量、歌词和播放进度，原歌曲保留。</p><button class="primary wide" :disabled="!versionName.trim()||!destination"><ListPlus :size="17"/>保存独立版本</button></form></template>
 <template v-else>
 <span class="modal-symbol"><Radio :size="25"/></span><h2>更换歌源</h2><p class="muted">{{song.artist}} · {{song.title}}<br/>综合评分：歌手 55% · 歌名 35% · 时长 10%。{{song.version?'按此版本类型匹配，可继续另存其他版本。':'官方标题加分，翻唱和乐器改编降权。'}}</p>
 <form class="source-search" @submit.prevent="search"><input v-model="keyword" class="field" aria-label="歌源搜索关键词"/><button class="secondary" :disabled="busy||!keyword.trim()"><Search :size="16"/>搜索</button></form>
 <p v-if="sourceSearchKeyword(song)!==sourceSearchKeyword(song,false)" class="muted">已识别片假名的英文拼写，可切换关键词：<button class="text-btn" :disabled="busy" @click="keyword=sourceSearchKeyword(song,false)">原名</button> · <button class="text-btn" :disabled="busy" @click="keyword=sourceSearchKeyword(song)">英文名</button></p>
 <p v-if="busy" class="muted"><LoaderCircle class="spin" :size="18"/>正在寻找候选音源…</p><p v-if="message" class="recognize-message" role="status">{{message.replace('BILIBILI_VERIFICATION_REQUIRED: ','')}}</p><button v-if="message.includes('BILIBILI_VERIFICATION_REQUIRED')" class="secondary wide" @click="emit('verify')">打开 B 站官方验证页面</button>
 <div class="source-candidates" role="radiogroup" aria-label="候选歌源"><div v-for="(t,i) in candidates" :key="t.bvid" :class="['source-option',{chosen:chosen===t.bvid}]"><label><input v-model="chosen" type="radio" name="source" :value="t.bvid"/><span><strong>{{t.title}}</strong><small>{{t.artist}} · {{t.duration||'未知时长'}} · {{i===0?'评分最高':`候选 ${i+1}`}}</small><small>{{t.reason}}</small></span></label><div class="source-option-actions"><b>{{t.score}}<small>分</small></b><button class="icon-btn" :title="`试听 ${t.title}`" @click="emit('preview',t)"><Play :size="16"/></button><button class="text-btn save-version" :aria-label="`将 ${t.title} 另存为独立版本`" @click="beginVersion(t)"><ListPlus :size="14"/>另存为独立版本</button></div></div></div>
 <button class="primary wide" :disabled="!chosen||busy" @click="confirm"><Check :size="17"/>使用选中的歌源</button>
 </template>
</template>
