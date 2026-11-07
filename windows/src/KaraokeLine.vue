<script setup lang="ts">
import { wordProgress,type LyricLine } from './lyrics'
defineProps<{line:LyricLine;position:number;offsetMs:number;active:boolean;past:boolean;translation:boolean}>()
</script>
<template>
 <span class="lyric-original"><template v-if="line.words?.length"><span v-for="(word,i) in line.words" :key="i" class="karaoke-word" :style="{'--word-progress':(active?wordProgress(word,position,offsetMs)*100:past?100:0)+'%'}">{{word.text}}</span></template><template v-else>{{line.text}}</template></span>
 <span v-if="translation&&line.translation" class="lyric-translation">{{line.translation}}</span>
</template>
