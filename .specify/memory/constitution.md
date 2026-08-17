<!--
Sync Impact Report
- Version change: N/A (初始创建, 0.0.0 -> 1.0.0)
- Modified principles: N/A (首次建立)
- Added sections: Core Principles (I-VI), Additional Constraints, Development Workflow, Governance
- Removed sections: N/A
- Deferred TODOs: 无
-->

# Czkawka Custom (中文重复照片筛查工具) Constitution
<!-- 本仓库为 qarmin/czkawka 的 fork，目标是定制一款中文界面的重复/相似照片筛查工具，
     核心新增能力："清晰度评分 + 自动建议保留最清晰照片 + 确认后处理"。 -->

## Core Principles

### I. 增量定制、保持上游可同步
本仓库是 czkawka 的 fork。所有改动必须遵循增量原则：只新增所需功能，不重写
上游成熟实现（哈希、分组、扫描、缓存、UI 框架）。核心改动集中在
`czkawka_core`（新增清晰度评分纯函数）与 `krokiet`（GUI 排序与交互）两层；
`czkawka_cli`、`czkawka_gui`、`cedinia` 保持与上游一致，不修改，以降低后续
同步上游的成本。

### II. 性能优先（继承上游）
扫描与文件操作必须快。沿用上游约束：rayon 并行、blake3/感知哈希等高效算法、
避免热路径上的不必要分配与拷贝。清晰度评分必须复用 `ImagesEntry` 已采集的
`width`/`height`/`size` 字段，禁止为评分重新解码图片。

### III. 数据安全默认（NON-NEGOTIABLE）
任何批量文件操作必须以"可恢复"为默认：未保留项默认移到系统回收站；永久删除
必须二次确认且为显式选择；移动到外部文件夹时保留原路径可追溯。绝不允许"一键
静默删除"。

### IV. 清晰度判定的确定性
"最清晰"必须有可测试、可复现的定义：第一排序键为总像素数（width×height），
第二排序键为文件大小（KB），完全相同时按路径字典序稳定排序。0 尺寸（解码失败）
条目排最后，绝不误标为建议保留。禁止使用不可靠的 EXIF 对焦/锐度信息。

### V. 中文界面完整性
最终交付为中文 GUI。新增的所有界面文案必须同时进入 `en` 与 `zh-CN` 翻译文件
（Fluent .ftl），保持语言切换完整；新增文案禁止硬编码在 Rust 或 Slint 中。

### VI. 质量纪律
TDD：先写测试再实现；执行质量门禁（`just fix`）；所有新代码无 clippy 警告；
测试覆盖评分规则边界（同分辨率、不同分辨率、0 尺寸、稳定排序）。完成实现后
必须实际构建并运行验证，禁止仅凭"代码写完"宣称完成。

## Additional Constraints

- 许可证：核心库 MIT；`krokiet`/`cedinia` GUI 为 GPL-3.0（上游既定，自用或公开
  源码均合规，不接受闭源再分发）。
- 技术栈：Rust（minimum 1.94.1, edition 2024）、Slint GUI、Fluent i18n。不引入
  非 Rust 原生依赖（libheif/libraw 仅经上游 optional feature 启用）。
- 独立工具：本定制不依赖用户其他项目，可独立构建、独立交付。
- 交付物：源码 + Windows 可执行文件（`target/release/krokiet.exe`）。

## Development Workflow

- 新功能开发统一走 Spec Kit 流程：`$speckit-specify` → `$speckit-plan` →
  `$speckit-tasks` → `$speckit-implement` → `$speckit-converge`。
- 规格/计划/任务产物统一存放在 `specs/<编号>-<功能名>/`（spec.md/plan.md/tasks.md）。
- 提交信息遵循仓库惯例（上游为英文，本 fork 兼容英文提交信息）。
- 合并前必须通过 `just fix` 质量门禁；测试用 `cargo test` 全绿。

## Governance

本宪法对所有开发活动生效，优先于其他任何非宪法约定。修正案需写入本文件并
升级版本号；所有实现必须验证符合本宪法原则。运行时开发指引以仓库根 AGENTS.md
及 `.specify/` 基础设施为准。

**Version**: 1.0.0 | **Ratified**: 2026-08-17 | **Last Amended**: 2026-08-17