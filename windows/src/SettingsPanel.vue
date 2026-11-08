<script setup lang="ts">
import { themes,type ThemeId } from './themes'
defineProps<{theme:ThemeId;insertOnClick:boolean;restoreProgress:boolean;defaultBalanceEnabled:boolean;userName?:string;searchCount:number}>()
const emit=defineEmits<{theme:[value:ThemeId];insert:[value:boolean];restore:[value:boolean];balance:[value:boolean];login:[];logout:[];recognize:[];clearSearch:[]}>()
</script>
<template>
 <h2>设置</h2><p class="settings-intro">设置自动保存，关闭面板后继续听歌。</p>
 <section class="settings-section"><h3>账号</h3><div class="settings-account"><span>{{userName||'尚未连接 B 站'}}<small>{{userName?'账号已连接':'连接后可使用账号有权限访问的资源'}}</small></span><button v-if="userName" class="secondary" @click="emit('logout')">退出登录</button><button v-else class="secondary" @click="emit('login')">登录 B 站</button></div></section>
 <section class="settings-section"><h3>播放与队列</h3><label class="settings-toggle"><span>点击歌曲插入当前队列<small>关闭后点击歌曲会替换队列，旧队列保留在历史中。</small></span><input type="checkbox" :checked="insertOnClick" @change="emit('insert',($event.target as HTMLInputElement).checked)"/></label><label class="settings-toggle"><span>打开后恢复播放进度<small>默认关闭，只恢复歌曲；开启后恢复时间点，两种模式都不会自动出声。</small></span><input type="checkbox" :checked="restoreProgress" @change="emit('restore',($event.target as HTMLInputElement).checked)"/></label><label class="settings-toggle"><span>全局音量平衡<small>统一开启或关闭所有跟随全局的歌曲；已自定义的歌曲保留自己的开关。</small></span><input type="checkbox" :checked="defaultBalanceEnabled" @change="emit('balance',($event.target as HTMLInputElement).checked)"/></label></section>
 <section class="settings-section"><h3>主题与强调色</h3><div class="settings-themes"><button v-for="t in themes" :key="t.id" :class="{selected:theme===t.id}" :aria-pressed="theme===t.id" @click="emit('theme',t.id)"><i :style="{background:t.accent}"/>{{t.name}}</button></div></section>
 <section class="settings-section"><h3>搜索与识曲</h3><div class="settings-account"><span>搜索历史<small>已保存 {{searchCount}} 条搜索词</small></span><button class="secondary" :disabled="!searchCount" @click="emit('clearSearch')">清空搜索历史</button></div><div class="settings-account"><span>听歌识曲<small>当前支持麦克风或文件，需要 AudD Token；系统音频回环采集尚未接入。</small></span><button class="secondary" @click="emit('recognize')">识曲配置</button></div></section>
</template>
