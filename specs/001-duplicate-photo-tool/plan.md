# Implementation Plan: 中文重复照片筛查工具

**Branch**: `001-duplicate-photo-tool` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-duplicate-photo-tool/spec.md`

## Summary

在 qarmin/czkawka 最新版 fork 上增量定制一款中文界面的重复/相似照片筛查工具。复用它成熟的
重复检测（哈希）与相似检测（感知哈希 + BKTree 分组）引擎，新增"清晰度评分"纯函数
（分辨率优先、文件大小次之），并在 Krokiet (Slint) GUI 中新增分组内清晰度排序、
⭐建议保留标记、手动覆盖、批量处理（默认回收站）交互。改动集中在 `czkawka_core`
（新模块）与 `krokiet`（UI + 翻译）两层，`czkawka_cli`/`czkawka_gui`/`cedinia` 保持
上游一致，便于后续同步。

## Technical Context

**Language/Version**: Rust 1.94.1+（workspace 既定，本机 rustc 1.97.1 / edition 2024）

**Primary Dependencies**: 复用 workspace 既有依赖——`image`、`image_hasher`、`rayon`、
`bk_tree`、`crossbeam-channel`、`slint` 1.17.0、Fluent i18n（`flk!` 宏）。新增逻辑不引入
新第三方依赖（评分只读 `ImagesEntry` 现有字段，满足宪法"最小非 Rust 依赖"）。

**Storage**: 沿用 czkawka 现有图像哈希缓存（缓存文件路径由 `get_similar_images_cache_file`
生成）；`ImagesEntry` 已持久化 `width`/`height`/`size`，评分数据无需额外存储。

**Testing**: `cargo test -p czkawka_core`（评分单元测试）、`cargo check/test -p krokiet`、
手工 GUI 冒烟测试（Windows）。

**Target Platform**: Windows 10+ x64（本机），Slint 跨平台特性保持其他平台可用。

**Project Type**: desktop-app（fork 改造 + 库模块新增）

**Performance Goals**: 扫描吞吐与上游持平（并行哈希）；评分过程 O(组内成员) 追加开销，
无新 IO；万张级照片一键扫描在分钟级完成。

**Constraints**: 不重写上游算法；不新增非 Rust 依赖；新增界面文案必须中英双语入 .ftl；
评分禁止重新解码图片；默认动作必须是可恢复路径。

**Scale/Scope**: 万张 ~ 十万张级照片目录；单一桌面应用；单机离线使用。

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 宪法条目 | 满足方式 |
|---|---|
| I. 增量定制、保持上游可同步 | 只改 core 新增模块 + krokiet 交互层；cli/gui/cedinia 不动 |
| II. 性能优先 | 评分复用已采集字段，无解码无 IO；不触碰热路径哈希 |
| III. 数据安全默认 | 默认回收站；永久删除二次确认；保留项绝不被处理 |
| IV. 清晰度判定的确定性 | 分数 = width×height×1000 + KB；0 尺寸排末；稳定排序 |
| V. 中文界面完整性 | 新增文案入 en/zh-CN .ftl，不硬编码 |
| VI. 质量纪律 | TDD 评分测试；`cargo test` 全绿；`just fix` 质量门禁；实测构建运行 |

GATE 判定：通过，无违规。

## Project Structure

### Documentation (this feature)

```text
specs/001-duplicate-photo-tool/
├── plan.md              # 本文件
├── research.md          # Phase 0 输出
├── data-model.md        # Phase 1 输出
├── quickstart.md        # Phase 1 输出
├── contracts/           # Phase 1 输出（核心 API 契约）
│   └── clarity-ranking.md
└── tasks.md             # $speckit-tasks 输出（后续生成）
```

### Source Code (仓库根)

```text
czkawka_core/src/tools/similar_images/
├── mod.rs               # 既有：ImagesEntry 定义（width/height/size 已齐）
├── core.rs              # 既有：哈希/分组引擎（不修改核心算法）
├── similarity_ranking.rs # 新增：清晰度评分纯函数 + 单元测试
└── similarity_ranking/tests.rs # 新增：评分边界测试

krokiet/src/
├── common.rs            # 既有：列索引常量（新增清晰度列索引）
├── connect_sort.rs      # 既有：排序回调（扩展支持按清晰度排序）
├── connect_scan.rs      # 既有：扫描回调（结果加载后调用评分排序）
├── file_actions/        # 既有：文件操作（复用删除/回收站能力）
└── connect_keep_best.rs # 新增：建议保留/手动覆盖/批量处理回调

krokiet/ui/              # Slint 界面声明
└── similar_images 相关   # 结果表格新增评分列/建议保留标记/操作按钮

krokiet/i18n/en/krokiet.ftl      # 新增文案（英文）
krokiet/i18n/zh-CN/krokiet.ftl   # 新增文案（简体中文）
```

**Structure Decision**: 遵循上游"core 引擎 + 前端"布局，本功能纵深不大，仅两个 crate
触及。新增文件均放在既有模块内，不新建顶层目录；译文按上游 i18n 惯例只新增 key。

## Complexity Tracking

> 宪法检查无违规，无需填写。