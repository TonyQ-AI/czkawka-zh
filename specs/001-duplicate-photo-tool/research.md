# Research: 中文重复照片筛查工具

**Phase 0 输出** | **2026-08-17**

## 决策汇总

| 待决项 | 决策 | 依据 |
|---|---|---|
| 检测引擎 | 复用 czkawka `similar_images` 模块（哈希 + 感知哈希 + BKTree 分组） | 上游成熟实现，32.7k stars；宪法要求不重写成熟算法 |
| 前端 | Krokiet (Slint GUI) | 官方主线、持续开发；用户已确认选择 |
| 评分数据来源 | 直接读 `ImagesEntry.width/height/size` 字段 | 已读源码确认字段在哈希阶段被填充（`core.rs:187-188`），零额外 IO |
| 评分算法 | `score = width * height * 1000 + size/1024` | 用户确认"分辨率优先、大小辅助"；无浮点避免不一致 |
| 默认处理动作 | 系统回收站（可恢复） | 宪法 III 数据安全默认 |
| 永久删除防护 | 显式勾选 + 二次确认对话框 | 用户确认；宪法 III |
| 新增依赖 | 无新第三方依赖 | 宪法 II（最小非 Rust 依赖）+ 成本最低 |
| 排序插入点 | Slint 表格模型（`SingleMainListModel` 的 val_int 列） | 已读 `krokiet/src/model_operations/` 结构，行模型支持任意 int 列 |
| 翻译机制 | Fluent .ftl 新增 key（en + zh-CN） | 上游 i18n 惯例，AGENTS.md 明确只改 en 源文件，非 en 语言文件由编译期/翻译流程维护 |

## 关键技术确认（直接代码结论）

1. **`ImagesEntry`（similar_images/mod.rs:34-42）**：`path/size/width/height/modified_date/hashes/difference`
   字段齐全，`Serialize/Deserialize` 驱动缓存持久化——评分所需数据天然可用。

2. **分组输出（similar_images/core.rs）**：`similar_vectors: Vec<Vec<ImagesEntry>>`，
   每个内层 Vec 是一个相似组；`similar_referenced_vectors` 为带参考条目的变体。
   评分排序作用于内层 Vec 即可，不触碰外层结构。

3. **GUI 行模型（krokiet/src/common.rs / shared_models.rs）**：`SingleMainListModel` 以
   `val_str: []string` + `val_int: []int` 扁平行表示，列索引常量定义在 `common.rs`
   （`StrDataSimilarImages` 系列）——新增"清晰度/建议保留"展示可为新列索引或复用现有列，
   具体在实现时按表格现有列布局决定。

4. **文件操作能力（krokiet/src/file_actions/）**：上游已实现移动到回收站/永久删除，
   定制直接复用这些底层操作，仅新增"收集未保留项清单"与确认流程。

5. **Windows 构建**：`cargo build --release -p krokiet`；本机 MSVC 工具链已确认可用
   （VS2022 BuildTools 14.44 + Windows SDK 10.0.26100），代理 7897 可拉取 crates。

## 备选方案评估（前期已与用户讨论）

| 方案 | 结论 | 原因 |
|---|---|---|
| 基于旧 GTK 版 czkawka 定制 | 否决 | 上游 12.0 起停止发布新二进制，无法跟进 |
| Czkawka CLI + 自写 GUI | 否决 | 功能落差大、性能边界差、工作量反而不小 |
| 基于 czkawka 新建独立顶层 crate | 否决 | 违反"增量定制"；core 模块内新增最贴合上游结构 |

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 首次编译 20-40 分钟 | 规划时预留；增量编译后续快 |
| Slint GUI 部分 GPL-3.0 | 用户已知悉；自用或公开源码合规 |
| 上游 UI 布局对新增列的适配 | 实现前先读目标表格 .slint 文件确认可扩展性 |
| 回收站 API 在 Windows 的可用性 | 复用上游 file_actions 已验证路径，避免自造轮子 |