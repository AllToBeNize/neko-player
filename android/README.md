# Android

Android 客户端预留目录。建议 Kotlin + Jetpack Compose + Media3。
与 Windows 共享歌单数据协议、匹配规则说明和测试样本，播放器与账号存储采用平台原生实现。

目标设备：iQOO Neo11；目标系统：OriginOS 7，完整构建版本待真机验证时记录。播放及媒体库采用 MediaLibraryService / MediaLibrarySession，预留标准系统播控和外部歌单浏览能力。
原子随身听公开专用 SDK 尚未查到，是否出现在其音源列表需要真机验证，详见 [适配调查](../docs/originos7-integration.md)。
