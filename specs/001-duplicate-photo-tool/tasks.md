---

description: "Task list for 中文重复照片筛查工具 feature implementation"
---

# Tasks: 中文重复照片筛查工具

**Input**: Design documents from `/specs/001-duplicate-photo-tool/`

**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: 本功能按宪法 VI 采用 TDD——测试任务先于实现，先验证失败再实现。

**Organization**: 按用户故事分组（US1-US4），每个故事可独立实现与测试。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可在并行执行（不同文件、无依赖）
- **[Story]**: 所属用户故事（US1-US4）
- 路径精确到文件

## Path Conventions

- czkawka_core：`czkawka_core/src/...`
- krokiet GUI：`krokiet/src/...`、`krokiet/ui/...`、`krokiet/i18n/<lang>/krokiet.ftl`
- 本功能纵深小，仅在既有 crate 内新增/修改，不新建顶层目录。

---

## Phase 1: Setup

**Purpose**: 仓库基线确认与功能分支

- [x] T001 确认工作目录 `E:\AiDatas\projects\czkawka-custom` 为 git 仓库，当前 HEAD 干净
- [x] T002 创建功能分支 `001-duplicate-photo-tool`（从当前 master/main 切出）
- [x] T003 验证本机构建基线：`cargo metadata --no-deps` 成功（Rust 1.97.1 满足 workspace 要求 1.94.1）
- [x] T004 [P] 确认代理可拉取 crates（`curl -x http://127.0.0.1:7897 https://crates.io` 返回非超时）

---

## Phase 2: Foundational（清晰度评分模块，阻塞所有故事）

**Purpose**: czkawka_core 新增评分纯函数——US2/US3/US4 的公共基础。完成前不得开始故事实现。

**?? CRITICAL**: 该模块是数据层基石，任何故事都依赖其存在。

### 测试（TDD——先写测试，确保 FAIL）

- [x] T005 [P] 在 `czkawka_core/src/tools/similar_images/similarity_ranking/tests.rs` 编写评分单元测试：不同分辨率排序（1000×800 > 800×600 > 500×400）
- [x] T006 [P] 在 `czkawka_core/src/tools/similar_images/similarity_ranking/tests.rs` 编写同分辨率看大小测试（体积大者优先）
- [x] T007 [P] 在 `czkawka_core/src/tools/similar_images/similarity_ranking/tests.rs` 编写 0 尺寸排末测试（width=0 或 height=0 → 得分 0）
- [x] T008 [P] 在 `czkawka_core/src/tools/similar_images/similarity_ranking/tests.rs` 编写稳定排序测试（同分按 path 字典序，结果可复现）与空组测试（keep_index=None）
- [x] T009 运行 `cargo test -p czkawka_core similar_images::similarity_ranking` 确认测试因模块缺失而失败（红）

### 实现

- [x] T010 创建模块文件 `czkawka_core/src/tools/similar_images/similarity_ranking.rs`，实现 `clarity_score(&ImagesEntry) -> u64`（分数 = width×height×1000 + size/1024；0 尺寸返回 0），契约见 `contracts/clarity-ranking.md`
- [x] T011 实现 `rank_group_by_clarity(&mut [ImagesEntry]) -> usize`（降序原地排序；同分按 path 升序；返回建议保留索引）、`group_keep_index(&[ImagesEntry]) -> Option<usize>`，契约见 `contracts/clarity-ranking.md`
- [x] T012 在 `czkawka_core/src/tools/similar_images/mod.rs` 中声明 `mod similarity_ranking;` 并 `pub use` 三个公共函数
- [x] T013 运行 `cargo test -p czkawka_core similar_images::similarity_ranking` 确认全部通过（绿）
- [x] T014 运行 `cargo clippy -p czkawka_core --all-targets` 确认无警告

**Checkpoint**: 评分模块可用——`cargo test -p czkawka_core` 全绿（含既有测试）

---

## Phase 3: User Story 1 - 扫描并找出重复与相似照片 (Priority: P1) ?? MVP

**Goal**: 用户选择文件夹扫描，得到重复/相似照片分组。上游 czkawka 已实现该引擎，本阶段为验证 + 配置默认参数（不漏报完全重复）。

**Independent Test**: 对含已知重复照片的测试目录运行扫描，结果正确分组且含完全重复组（quickstart.md 步骤 3）。

### 实现

- [x] T015 审阅 `czkawka_core/src/tools/similar_images/core.rs` 确认分组输出路径（similar_vectors / similar_referenced_vectors），记录字段含义——不改代码，仅输出审阅笔记到 `specs/001-duplicate-photo-tool/notes-engine-review.md`
- [x] T016 [P] [US1] 检查 krokiet 相似图片设置页默认参数（similarity preset = High、排除同大小/同分辨率默认关闭），确保默认扫描可发现"完全不同名但内容相同"的照片，必要时在 `krokiet/ui/screens/tool_settings.slint` 调整默认值
- [x] T017 [US1] 构建并运行 `cargo run -p krokiet`，实测扫描含 2 张完全相同（异名异大小）+ 独有图的目录，确认分组正确（对应 quickstart 步骤 3 断言 1/4）

**Checkpoint**: US1 完成——扫描分组功能经 GUI 实测验证

---

## Phase 4: User Story 2 - 自动建议保留最清晰的一张 (Priority: P1)

**Goal**: 每个重复组内按清晰度排序并自动标记 ⭐ 建议保留最清晰者。

**Independent Test**: 含 3 个分辨率版本的组，建议保留标记落在最高分辨率文件上（quickstart 步骤 3 断言 2/3/5）。

### 实现

- [x] T018 [P] [US2] 在 `krokiet/src/model_operations/model_processor.rs` 中定位 Similar Images 行模型构建处，将每组行数据（header + 成员）调用 `rank_group_by_clarity` 排序后写入模型（保持组结构不变）
- [x] T019 [P] [US2] 在 krokiet 相似图片结果表 Slint 声明（`krokiet/ui/screens/main_lists.slint` 或对应组件）中新增"清晰度"列展示（复用已有 PixelCount/Width/Height 列数据或新增列索引，取实现成本低者）
- [x] T020 [US2] 新增"建议保留"标记逻辑：每组排序后第 0 行默认显示 ⭐，通过 `SingleMainListModel` 新增布尔列或在既有列标记——在 `krokiet/src/common.rs` 扩展列枚举
- [x] T021 [US2] 在 `krokiet/i18n/en/krokiet.ftl` 与 `krokiet/i18n/zh-CN/krokiet.ftl` 新增文案：`similar_images_keep_best`（建议保留）、`similar_images_keep_best_tooltip`（保留最清晰的一张）
- [x] T022 [US2] 构建 `cargo build -p krokiet` 并运行，实测含 3 分辨率版本的组：排序正确 + ⭐ 落在最高分辨率行（quickstart 步骤 3 断言 2）

**Checkpoint**: US2 完成——自动排序与建议保留标记在 GUI 上可见

---

## Phase 5: User Story 3 - 手动覆盖保留选择 (Priority: P2)

**Goal**: 用户点选组内任意行可覆盖 ⭐ 建议保留标记。

**Independent Test**: 点击组内另一行，⭐ 标记移动（quickstart 步骤 4）。

### 实现

- [x] T023 [P] [US3] 复用上游勾选机制实现手动覆盖：升级 `SelectAllExceptBiggestResolution` 为清晰度语义（分辨率优先+大小次之，`krokiet/src/connect_select/mod.rs` 的 `extract_comparable_field`），用户点选按钮即采纳建议保留，逐行勾选/取消即手动覆盖
- [x] T024 [US3] 边界处理：同一组内勾选覆盖后自动建议行保持未勾选；跨组操作互不影响（上游 find_header_idx_and_deselect_all 按组分组处理天然隔离）；取消全部勾选=保留全部（无待处理项，批量按钮自动禁用）
- [x] T025 [US3] 双语更新选择按钮文案：`selection_the_biggest_resolution`（选择最清晰一张，最高分辨率优先）与 `selection_all_except_biggest_resolution`（保留最清晰一张，选择其余所有项），`krokiet/i18n/{en,zh-CN}/krokiet.ftl`
- [x] T026 [US3] 编译通过（cargo check -p krokiet 零警告）；手动覆盖路径 = 按钮一键采纳 + 逐行勾选覆盖，GUI 冒烟测试留待 Polish（T038）

**Checkpoint**: US2+US3 均可独立工作——标记可自动可手动

---

## Phase 6: User Story 4 - 确认后统一处理（默认回收站） (Priority: P2)

**Goal**: 批量处理未保留项——默认回收站；可选移文件夹；永久删除双重确认。

**Independent Test**: 移到回收站后文件可恢复；永久删除被双重确认拦截（quickstart 步骤 5-6）。

### 实现

- [x] T027 [P] [US4] 审阅 `krokiet/src/file_actions/connect_delete.rs` 与 `connect_move.rs` 的既有实现，确认回收站/移动能力可复用，输出复用方案至 `specs/001-duplicate-photo-tool/notes-file-actions.md`
- [x] T028 [US4] 在 krokiet 相似图片页操作区（`krokiet/ui/screens/action_buttons.slint`）新增按钮组："移到回收站"（默认）、"移到文件夹"、"永久删除"，可见性绑定收集到的未保留项数量
- [x] T029 [US4] 实现批量处理回调（`krokiet/src/connect_keep_best/` 新建）：收集当前模型中被标记为保留（⭐）项以外的成员路径清单，调用 `ProcessFunction::Simple` 执行回收站/移动/删除（复用 `model_processor.rs` 的 `process_and_update_gui_state`）
- [x] T030 [US4] 永久删除防护：按钮默认禁用，需勾选"我已确认"复选框 + 弹确认弹窗（复用 `krokiet/ui/popups/` 既有 popup 模式 `ConfirmPopup`）二次确认后才执行
- [x] T031 [US4] 处理前展示文件清单：弹窗列出将处理的 N 个文件（截断展示 + 完整路径可展开），用户确认后执行
- [x] T032 [US4] 失败处理：单项失败记录错误信息（复用 Messages 机制），完成后提示"成功 N / 失败 M / 释放 X MB"
- [x] T033 [US4] 提高每次处理后的模型清理：成功项从模型移除，保留项保留（复用 `remove_processed_items_from_model`）
- [x] T034 [US4] 在 `krokiet/i18n/en/krokiet.ftl` 与 `krokiet/i18n/zh-CN/krokiet.ftl` 新增文案：`move_to_recycle_bin`、`move_to_folder`、`delete_permanently`、`confirm_i_understand`、`confirm_delete_warning`、`processed_summary`
- [x] T035 [US4] 构建并实测全流程：扫描→确认保留→移回收站→验证可恢复、永久删除双重确认、失败项提示（quickstart 步骤 5-6）

**Checkpoint**: US4 完成——闭环打通，数据安全默认生效

---

## Phase 7: Polish & Cross-Cutting

**Purpose**: 质量门禁、翻译完整性与文档收尾

- [x] T036 [P] 运行 `cargo test -p czkawka_core` 全量测试确认全绿（含既有 + 新增）
- [x] T037 [P] 运行 `just fix` 质量门禁（格式化 + clippy + Python 检查），修复所有输出直至零错误
- [x] T038 执行 `specs/001-duplicate-photo-tool/quickstart.md` 全部验证步骤，对照 SC-001 至 SC-007 逐项确认
- [x] T039 检查 `krokiet/i18n/zh-CN/krokiet.ftl` 覆盖新增 key 无遗漏（对照 en 文件 diff）
- [x] T040 [P] 更新 `docs/superpowers/specs/2026-08-17-zh-duplicate-photo-tool-design.md` 或注释标记"已按 spec-kit 流程实现"（若该文件仍需保留）
- [x] T041 提交最终变更，写清变更说明（新增模块、GUI 交互、翻译、测试结果）

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖，立即开始
- **Foundational (Phase 2)**: 依赖 Phase 1 完成——阻塞所有用户故事
- **US1 (Phase 3)**: 依赖 Phase 2（评分模块存在以支持后续展示；US1 本身只验证扫描分组）
- **US2 (Phase 4)**: 依赖 Phase 2 + US1 的引擎审阅结论
- **US3 (Phase 5)**: 依赖 US2（标记机制）——可并行于 US4
- **US4 (Phase 6)**: 依赖 US2/US3 的标记逻辑——可并行于 US3 的部分任务
- **Polish (Phase 7)**: 依赖所有期望的故事完成

### User Story Dependencies

- **US1 (P1)**: Phase 2 后即可开始，无其他故事依赖
- **US2 (P1)**: 依赖 Phase 2 + US1
- **US3 (P2)**: 依赖 US2
- **US4 (P2)**: 依赖 US2/US3

### Within Each Story

- 测试先写、先失败（TDD）→ 实现 → 验证
- 模型/核心逻辑先于 UI 交互
- 故事完成后再进入下一优先级

### Parallel Opportunities

- Phase 1 的 T004 可与 T001-T003 并行
- Phase 2 测试任务 T005-T008 全部并行（不同测试文件内不同函数）
- US3 与 US4 的部分任务（T024/T030 不同文件）可并行
- Polish 的 T036/T037/T039/T040 并行

---

## Parallel Example: Phase 2 测试编写

```bash
# 并行运行全部评分测试（先失败）：
Task: "T005 不同分辨率排序测试"
Task: "T006 同分辨率看大小测试"
Task: "T007 0 尺寸排末测试"
Task: "T008 稳定排序 + 空组测试"
```

---

## Implementation Strategy

### MVP First

1. Phase 1: Setup
2. Phase 2: Foundational（评分模块）—— CRITICAL，阻塞一切
3. Phase 3: US1 —— 验证扫描分组（引擎已有，验证即 MVP 核心）
4. **STOP and VALIDATE**: 对照 quickstart 步骤 3 验证扫描分组
5. 继续 US2 → US3 → US4 增量交付

### Incremental Delivery

1. Setup + 评分模块 → 基础就绪
2. US1 扫描分组 → 可识别重复（MVP）
3. US2 自动建议保留 → 可看出该留哪张
4. US3 手动覆盖 → 用户有最终控制权
5. US4 批量处理 → 闭环（回收站默认安全）

---

## Notes

- 所有新增界面文案必须中英双语（en + zh-CN），禁止硬编码
- 处理操作默认走回收站；永久删除必须双重确认（宪法 III 强制）
- 评分逻辑禁止重新解码图片（复用 ImagesEntry 字段，契约强制）
- 提交粒度：每个可编译状态或逻辑组提交一次；提交信息用英文
- 完成宣称前必须跑验证命令（cargo test / 构建 / GUI 实测），禁止仅凭代码写完宣称完成
