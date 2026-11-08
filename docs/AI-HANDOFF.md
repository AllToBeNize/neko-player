# AI 交接说明

更新日期：2026-11-08。接手时先读本文件、根目录 AGENTS.md、windows/README.md；详细设计在 docs/architecture.md。

## 运行与目录

- 仓库： https://github.com/AllToBeNize/neko-player ，Windows 代码在 windows/，Android 仅占位与方案，尚未开发客户端。
- Windows：Tauri 2 + Rust + Vue 3 + TypeScript + Vite，WebView2 音频，Rust 本机媒体代理。
- 启动：在 windows/ 执行 npm install、npm run desktop。需要 Rust MSVC、VS C++ Build Tools、WebView2。desktop 脚本补充 Cargo PATH。
- 前端构建：npm run build --prefix windows。Rust 检查：cargo check --manifest-path windows/src-tauri/Cargo.toml。
- 当前使用开发窗口，没有生成正式安装包；无托盘、完整系统媒体键或独立原生音频引擎。
- SQLite 在 %APPDATA%/com.alltobenize.neko-player/library.sqlite；登录 Cookie 和 AudD Token 在 Windows 凭据管理器。不要把真实数据库、凭据、日志提交或输出。

## 已实现的功能

| 功能 | 当前行为与入口 |
| --- | --- |
| 酷狗歌单 | 支持官方分享短链接、完整分页读取，保留歌名/歌手/hash/封面/时长；用户实际歌单曾核验 1091 首。 |
| B 站登录 | 扫码，以及独立官方网页登录窗口的密码/短信入口；验证会话保存后自动关闭窗口。 |
| 搜索播放 | 歌名/歌手、BV 号与视频链接，首个分 P 音轨；暂停、跳转、音量、四种播放模式。 |
| 音源匹配 | 歌手55/歌名35/时长10，官方标题加分、版本不一致降权；默认选择最高候选，可人工换源。 |
| 跨文字名称 | 歌名主体、括号英文别名、有限片假名外来语词典及歌手别名；不是通用翻译模型。 |
| 独立版本 | 候选旁「另存为独立版本」，命名并选择歌单；新身份独立音量、歌词和进度，原歌不变。 |
| 歌单/收藏 | 本地歌单管理、加入目标歌单、收藏与最近播放。 |
| 搜索推荐 | 最近搜索、本地曲库、音乐方向词及在线联想，去重、防抖与过期请求保护。 |
| 队列 | 默认插入播放，可在设置改为替换；插队标记、每项进度、最多10个历史会话。 |
| 队列编辑 | 拖手柄排序；右键或行内更多：播放、下一首、加入歌单、换源、上下移动、队首、移出队列。 |
| 底部功能 | 当前歌曲旁更多按钮复用队列功能菜单；封面展开详情与歌词。 |
| 启动恢复 | 默认恢复到歌曲、0秒、暂停；可开启恢复秒数，仍不自动播放。 |
| 歌词 | KRC真实字/音节时间填色、LRC逐行跟随、点击跳转、偏移、导入、歌词已有译文及手动译文。 |
| 音量 | 全局主音量；每首歌独立记忆平衡开关与 -12～+12dB 手动增益，支持数字和滑块。 |
| 主题设置 | 左下角账号区域、顶部调色盘打开设置；账号、主题、播放/队列、默认平衡、搜索历史和识曲入口集中在此。 |
| 识曲 | 已有麦克风录制最多12秒、文件片段、AudD配置及识别后搜索入口。真实Token识别未验收。 |

## 重要限制与后续工作

1. 用户实际要识别电脑正在播放的音频。WASAPI Loopback 尚未实现，不能把当前麦克风功能说成系统音频采集；下一步应加 Windows 输出设备回环捕获，再交给识曲服务。
2. AudD 需用户自己的 Token，存在服务额度/费用；不支持哼唱。不要声称已有无 Token 识曲或离线识曲。
3. 翻译来自歌词内附译文或用户导入；无机器翻译。无逐字时间轴时只做逐行，不用平均字长假装演唱对齐。
4. B站 code=0/data.v_voucher 也是验证要求；首次发现即暂停搜索并持久化，通过官方页面由用户完成。不能靠匿名切换、轮换账号规避验证。
5. 官方字样是标题线索，不是已核验账号认证；完整1091首匹配准确率没有验收。接口会变化。
6. 本地文件保存元数据/队列，重启需重新选择文件恢复授权；不能持久保存 blob URL。
7. Android/iQOO Neo11 OriginOS7 原子随身听调研见 docs/originos7-integration.md，不要当作已实现集成。

## 文件与数据约定

- App.vue：主状态、播放协调、IPC、曲库保存、队列与菜单、设置连接。目前较大，后续可谨慎拆分，先保住持久化和异步代次保护。
- SettingsPanel.vue：受控设置 UI；SourcePicker.vue：候选、评分、独立版本保存表单；TrackDetails.vue：歌词和逐曲音量。
- matching.ts / music-names.ts：MATCH_VERSION=4、版本意图、别名和搜索构造。manual 映射不可被升级自动覆盖。
- track-versions.ts：version:UUID 独立身份，根 parentId/baseTitle、referenceTitle、searchTerms；不复制旧 hash/歌词/偏移/URL。
- track-audio.ts：trackAudioSettings 稳定键；酷狗按hash共用，独立版本按新ID隔离。
- audio-balance.ts：分析→自动增益→手动增益→压缩→主音量；手动增益不能被自动平衡抵消。不是 LUFS 标准化或真峰值保证。
- queue-history.ts：entryId 区别重复歌曲。cursorId/positions/pending 都按 entryId；排序不换ID、不重播。下一首复用队列项，不复制。
- 最后一项被用户移除时保留 cleared=true 的空会话；compactSessions 保留该显式空状态，避免启动从历史误恢复。不要删除这个标记或放宽意外空队列保护。
- Rust playback_store：CAS revision + SQLite immediate transaction，禁止旧状态覆盖，保留5份备份。读取失败不允许用空数据自动覆盖。
- lyrics.ts / KaraokeLine.vue：字时钟、文件offset、译文；Rust krc.rs / lyrics.rs：受限解压与独立歌词缓存。
- Rust main.rs / web_login.rs / search_response.rs / search_verification.rs / kugou.rs / suggestions.rs：平台服务与代理。

## 验证与协作要求

- 用户要求：所有验证脚本/测试/截图必须位于项目外专用 TEMP 目录，完成后删除；不得提交这些文件。产品源码和技术文档可保留。
- reference/ 是只读 GPL 参考库，根项目为 MIT，不得直接复制 GPL 代码。
- 本轮队列使用临时 Playwright + 模拟 IPC 验证真实拖动、重复歌曲准确定位、菜单、移除、空队列重载；不是对真实平台全链路的验收。
- 以往已验证真实 KRC 字时间和译文、实际 Web Audio 正负增益；其余依照 windows/README.md 的明确范围，不夸大实测。
- 不重置用户数据库，不提交真实账号信息，不自行覆盖手动歌源。新增功能保持中文交互、强对比强调色。
