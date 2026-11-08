import type { TrackVersion } from './track-versions'
// Conservative loanword aliases: only fully covered katakana names become English aliases.
// This is spelling equivalence, not translation of arbitrary Japanese song titles.
const loanwords:Record<string,string>={
 'ブランド':'brand','ブラン':'brand','ニュー':'new','ニュウ':'new','デイ':'day','デー':'day',
 'ラブ':'love','ラヴ':'love','ラヴァー':'lover','ラバー':'lover','ユー':'you','ユア':'your',
 'マイ':'my','ミー':'me','アイ':'i','アイズ':'eyes','アイム':'im','ウィー':'we','アワー':'our',
 'ザ':'the','ジ':'the','オブ':'of','フォー':'for','トゥ':'to','イン':'in','オン':'on','アンド':'and',
 'ナイト':'night','ナイツ':'nights','ライト':'light','ライツ':'lights','ライフ':'life','タイム':'time',
 'ワールド':'world','ドリーム':'dream','ドリームズ':'dreams','スター':'star','スターズ':'stars',
 'スカイ':'sky','ブルー':'blue','レッド':'red','ブラック':'black','ホワイト':'white','ピンク':'pink',
 'ハート':'heart','ハーツ':'hearts','ソウル':'soul','ミュージック':'music','ソング':'song',
 'ダンス':'dance','ダンシング':'dancing','スマイル':'smile','ハッピー':'happy','サニー':'sunny',
 'サマー':'summer','ウィンター':'winter','スプリング':'spring','レイン':'rain','スノー':'snow',
 'ビューティフル':'beautiful','ビューティー':'beauty','スイート':'sweet','スウィート':'sweet',
 'ストーリー':'story','メモリー':'memory','メモリーズ':'memories','メロディー':'melody','メロディ':'melody',
 'フラワー':'flower','フラワーズ':'flowers','チェリー':'cherry','ポップ':'pop','シュガー':'sugar',
 'バタフライ':'butterfly','サンシャイン':'sunshine','ムーンライト':'moonlight','サンライズ':'sunrise',
 'ネバー':'never','エバー':'ever','フォーエバー':'forever','アゲイン':'again','オール':'all',
 'ワン':'one','ツー':'two','スリー':'three','ファースト':'first','ラスト':'last','オンリー':'only',
 'ガール':'girl','ガールズ':'girls','ボーイ':'boy','ボーイズ':'boys','ベイビー':'baby',
 'シークレット':'secret','マジック':'magic','ミラクル':'miracle','エンジェル':'angel',
 'ブレイク':'break','ブロークン':'broken','グラス':'glass','ビート':'beat','ビーツ':'beats',
 'フリー':'free','フリーダム':'freedom','トゥモロー':'tomorrow','ハロー':'hello','グッバイ':'goodbye',
 'テイク':'take','ミス':'miss','キス':'kiss','キル':'kill','フィール':'feel','フィーリング':'feeling'
}
const tokens=Object.keys(loanwords).sort((a,b)=>b.length-a.length)
export function englishLoanword(value:string):string|undefined{
 const kana=value.normalize('NFKC').replace(/[\s・･\-‐―]/g,'')
 if(!/^[ァ-ヶー]+$/.test(kana))return
 const memo=new Map<number,string|undefined>()
 function read(index:number):string|undefined{if(index===kana.length)return '';if(memo.has(index))return memo.get(index);for(const token of tokens){if(kana.startsWith(token,index)){const rest=read(index+token.length);if(rest!==undefined){const result=loanwords[token]+(rest?' '+rest:'');memo.set(index,result);return result}}}memo.set(index,undefined);return}
 return read(0)
}
const artistGroups=[['安田レイ','安田丽','安田麗','Rei Yasuda','Yasuda Rei']]
export function artistAliases(value:string):string[]{
 const names=value.split(/[、,，&/／;；]/).flatMap(name=>[name.replace(/[（(][^）)]*[）)]/g,''),...Array.from(name.matchAll(/[（(]([^）)]+)[）)]/g),m=>m[1])]).map(s=>s.trim()).filter(Boolean)
 const key=(s:string)=>s.normalize('NFKC').toLowerCase().replace(/[^\p{L}\p{N}]/gu,'')
 for(const group of artistGroups)if(names.some(name=>group.some(alias=>key(alias)===key(name))))names.push(...group)
 return [...new Set(names)]
}
export function titleAliases(value:string):{text:string;kind:'original'|'bracket'|'loanword'}[]{
 const original=value.normalize('NFKC').trim(),base=original.replace(/[（(][^）)]*[）)]/g,'').trim()
 const variants:{text:string;kind:'original'|'bracket'|'loanword'}[]=[{text:original,kind:'original'}]
 if(base&&base!==original)variants.push({text:base,kind:'original'})
 for(const m of original.matchAll(/[（(]([^）)]+)[）)]/g))if(/^[a-z\d\s'-]+$/i.test(m[1])&&!/^(live|cover|remix|instrumental|ver|version|short|full|off vocal)\b/i.test(m[1]))variants.push({text:m[1],kind:'bracket'})
 for(const item of [...variants]){const english=englishLoanword(item.text);if(english)variants.push({text:english,kind:'loanword'})}
 return variants
}
// Subtitle translations are useful ranking aliases, but over-constrain platform search.
export function preferredSearchTitle(value:string):string{
 const normalized=value.normalize('NFKC').trim()
 const version=/钢琴|鋼琴|吉他|小提琴|大提琴|伴奏|人声|现场|演唱会|翻唱|混音|加速|变速|降调|升调|原版|ピアノ|ギター|ライブ|\blive\b|\bcover\b|\bremix\b|\bpiano\b|\bguitar\b|\bacoustic\b|\binstrumental\b|\bversion\b|\bver\.?|\bfeat\.?|\boff\s*vocal\b/i
 const cleaned=normalized.replace(/\(([^)]+)\)/g,(whole,content:string)=>version.test(content)?whole:'').replace(/\s+/g,' ').trim()
 return cleaned||normalized
}
export function sourceSearchKeyword(song:{artist:string;title:string;version?:TrackVersion},english=true):string{
 const baseTitle=preferredSearchTitle(song.version?.baseTitle||song.title)
 const alias=englishLoanword(baseTitle)
 const title=english&&alias?alias:baseTitle
 const intent=song.version?(song.version.searchTerms.length?song.version.searchTerms.join(' '):song.version.name):''
 return [song.artist,title,intent].map(s=>s.normalize('NFKC').replace(/\s+/g,' ').trim()).filter(Boolean).join(' ')
}

export function sourceSearchQueries(song:{artist:string;title:string;version?:TrackVersion}):string[]{
 const base=preferredSearchTitle(song.version?.baseTitle||song.title)
 const artists=artistAliases(song.artist).filter(name=>name!==song.artist).slice(0,2)
 const withName=(artist:string,title:string)=>sourceSearchKeyword({...song,artist,title,version:song.version?{...song.version,baseTitle:title}:undefined})
 const queries=[sourceSearchKeyword(song),sourceSearchKeyword(song,false),...artists.map(name=>withName(name,base)),withName('',base)]
 const suffix=Array.from(base.matchAll(/\([^)]*\)/g),m=>m[0]).join(' ')
 for(const alias of titleAliases(song.version?.baseTitle||song.title)){const name=preferredSearchTitle(alias.text);if(name&&name!==base)queries.push(withName(song.artist,[name,suffix].filter(Boolean).join(' ')))}
 const seen=new Set<string>();return queries.map(q=>q.trim()).filter(q=>{const key=q.toLowerCase().replace(/\s+/g,' ');if(!key||seen.has(key))return false;seen.add(key);return true}).slice(0,5)
}
