<script setup lang="ts">
import {ref,computed,watch,onUnmounted,useId,nextTick}from 'vue'
import {invoke}from '@tauri-apps/api/core'
import {Search,Clock3,Music2,ArrowUpLeft,LoaderCircle}from 'lucide-vue-next'
import {buildSuggestions}from './search-suggestions'
const props=withDefaults(defineProps<{modelValue:string;history:string[];library:{title:string;artist:string}[];wide?:boolean;busy?:boolean;label?:string}>(),{label:'搜索音乐'})
const emit=defineEmits<{'update:modelValue':[value:string];search:[value:string];clearHistory:[]}>()
const wrapper=ref<HTMLElement>(),opened=ref(false),index=ref(-1),remote=ref<string[]>([]),loading=ref(false),composition=ref(false),unavailable=ref(false),id=useId()
const suggestions=computed(()=>buildSuggestions(props.modelValue,props.history,props.library,remote.value))
let timer:ReturnType<typeof setTimeout>|undefined,blurTimer:ReturnType<typeof setTimeout>|undefined,generation=0,alive=true
function close(){opened.value=false;index.value=-1;generation++;if(timer)clearTimeout(timer);loading.value=false}
function refresh(){if(timer)clearTimeout(timer);const run=++generation;remote.value=[];index.value=-1;unavailable.value=false;const q=props.modelValue.trim();if(!opened.value||composition.value||!q||/^https?:\/\//i.test(q)||/^BV/i.test(q)){loading.value=false;return}loading.value=true;timer=setTimeout(async()=>{try{const words=await invoke<string[]>('search_suggestions',{keyword:q});if(alive&&run===generation)remote.value=words}catch{if(alive&&run===generation)unavailable.value=true}finally{if(alive&&run===generation)loading.value=false}},320)}
function focus(){if(blurTimer)clearTimeout(blurTimer);opened.value=true;refresh()}
function input(e:Event){opened.value=true;emit('update:modelValue',(e.target as HTMLInputElement).value)}
function blur(){blurTimer=setTimeout(()=>close(),140)}
function submit(value?:string){if(composition.value)return;const query=(value??props.modelValue).trim();if(!query)return;close();emit('update:modelValue',query);emit('search',query)}
function keydown(e:KeyboardEvent){if(composition.value||e.isComposing)return;if(e.key==='ArrowDown'||e.key==='ArrowUp'){e.preventDefault();if(!opened.value)focus();const count=suggestions.value.length;if(count)index.value=index.value<0?(e.key==='ArrowDown'?0:count-1):(index.value+(e.key==='ArrowDown'?1:-1)+count)%count}else if(e.key==='Enter'){e.preventDefault();e.stopPropagation();submit(opened.value&&index.value>=0?suggestions.value[index.value]?.text:undefined)}else if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close()}}
watch(()=>props.modelValue,()=>refresh())
watch(index,async i=>{if(i<0)return;await nextTick();document.getElementById(id+'-item-'+i)?.scrollIntoView({block:'nearest'})})
onUnmounted(()=>{alive=false;generation++;if(timer)clearTimeout(timer);if(blurTimer)clearTimeout(blurTimer)})
</script>
<template>
 <div ref="wrapper" :class="['search-input-wrap',{wide}]" @focusout="blur">
  <form :class="wide?'discovery-search-form':'searchbox'" @submit.prevent="submit()"><Search :size="wide?21:17"/><input :value="modelValue" :aria-label="label" role="combobox" aria-autocomplete="list" :aria-expanded="opened" :aria-controls="id+'-suggestions'" :aria-activedescendant="opened&&index>=0?id+'-item-'+index:undefined" autocomplete="off" :placeholder="wide?'输入歌名、歌手、BV 号或 B 站视频链接':'歌名、歌手、BV 号或视频链接…'" @input="input" @focus="focus" @keydown="keydown" @compositionstart="composition=true" @compositionend="composition=false;refresh()"/><button v-if="wide" class="primary" :disabled="busy">{{busy?'搜索中…':'搜索音乐'}}</button><kbd v-else>↵</kbd></form>
  <section v-if="opened" :id="id+'-suggestions'" class="search-suggestion-panel" role="listbox" aria-label="搜索推荐" @mousedown.prevent><header><span>{{modelValue.trim()?'搜索联想':'最近搜索与音乐推荐'}}</span><LoaderCircle v-if="loading" class="spin" :size="13"/><button v-if="!modelValue.trim()&&history.length" type="button" @click="emit('clearHistory')">清空历史</button></header><button v-for="(item,i) in suggestions" :id="id+'-item-'+i" :key="item.text" type="button" role="option" :aria-selected="index===i" :class="['search-suggestion',{highlighted:index===i}]" @mouseenter="index=i" @click="submit(item.text)"><Clock3 v-if="item.kind==='history'" :size="15"/><Music2 v-else-if="item.kind==='library'||item.kind==='music'" :size="15"/><Search v-else :size="15"/><span>{{item.text}}<small>{{item.label}}</small></span><ArrowUpLeft :size="14"/></button><footer v-if="unavailable">在线联想暂不可用，仍可搜索或选择本地推荐</footer><footer v-else>↑↓ 选择 · Enter 搜索 · Esc 收起</footer></section>
 </div>
</template>
