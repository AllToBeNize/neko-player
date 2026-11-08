<script setup lang="ts">
import { ref,watch } from 'vue'
import { Music2 } from 'lucide-vue-next'
const props=defineProps<{src?:string}>()
const failed=ref(false)
watch(()=>props.src,()=>failed.value=false)
</script>
<template><span class="queue-cover"><img v-if="src&&!failed" :src="src" alt="" draggable="false" loading="lazy" referrerpolicy="no-referrer" @error="failed=true"/><Music2 v-else :size="21"/></span></template>
