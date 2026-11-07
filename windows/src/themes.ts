export const themes=[
 {id:'forest',name:'翡翠绿',accent:'#087f5b',strong:'#056449',soft:'#dbf2e7',page:'#fafcfb',side:'#edf3ef'},
 {id:'ocean',name:'海盐蓝',accent:'#205ecb',strong:'#174aa4',soft:'#e0eaff',page:'#fafbfe',side:'#eef2f9'},
 {id:'iris',name:'鸢尾紫',accent:'#7540bd',strong:'#5a289c',soft:'#ece2fb',page:'#fcfafe',side:'#f2eef7'},
 {id:'rose',name:'朱砂红',accent:'#b13e4d',strong:'#922c3b',soft:'#f9e0e4',page:'#fefafb',side:'#f7eef0'}
] as const
export type ThemeId=typeof themes[number]['id']
export function applyTheme(id:ThemeId){const t=themes.find(t=>t.id===id)||themes[0];const root=document.documentElement;for(const [key,value]of Object.entries({accent:t.accent,'accent-strong':t.strong,'accent-soft':t.soft,'page-bg':t.page,'side-bg':t.side})){root.style.setProperty(`--${key}`,value)}root.dataset.theme=t.id}
