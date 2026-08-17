# Data Model: 中文重复照片筛查工具

**Phase 1 输出** | **2026-08-17**

数据模型全部复用上游既有结构，本功能**不新增持久化实体**，仅新增"清晰度得分"这一派生值。

## 核心实体：ImagesEntry（上游既有，czkawka_core/src/tools/similar_images/mod.rs:34）

```rust
pub struct ImagesEntry {
    pub path: PathBuf,        // 文件绝对路径
    pub size: u64,            // 文件大小（字节）
    pub width: u32,           // 图片宽（像素）
    pub height: u32,          // 图片高（像素）
    pub modified_date: u64,   // 修改时间戳
    pub hashes: Vec<ImHash>,  // 感知哈希（1-4 个，按 geometric_invariance）
    pub difference: u32,      // 与参考条目的汉明距离
}
```

**状态来源**：哈希阶段由 `get_dynamic_image_from_path` 解码后填充 width/height（core.rs:187-188），
size 来自文件系统。**评分所需字段天然已就绪。**

## 派生值：清晰度得分（本功能新增，不持久化）

```
clarity_score = (width as u64) * (height as u64) * 1000 + (size / 1024)
```

- 类型：`u64`（避免浮点与溢出；400MP 图约 4×10^14，u64 安全）
- 排序方向：降序（得分高者更清晰，排在组内最前）
- 边界：width=0 或 height=0 → 得分 0，排组末，绝不被自动标记为建议保留
- 完全同分（含同图多尺寸同像素但体积相同）→ 按 path 字典序稳定排序保证确定性

## 分组结构（上游既有输出）

| 输出 | 类型 | 用途 |
|---|---|---|
| `similar_vectors` | `Vec<Vec<ImagesEntry>>` | 每个内层 Vec 为一个相似组（无参考模式） |
| `similar_referenced_vectors` | `Vec<(ImagesEntry, Vec<ImagesEntry>)>` | 参考-成员模式（GUI 实际采用） |

**GUI 行模型（SingleMainListModel）中的组表示**：上游以"header 行 + 成员行"扁平排布，
`checked` 字段标记选中（删除/移动目标）。本功能沿用该表示：
- header 行 = 参考条目本身（filled_header_row）
- 成员行 = 同组其他条目
- 组内排序 = 按清晰度对"header + 成员"整体重排的展示顺序

## 状态流转（处理动作）

```
扫描完成 → 每组按清晰度排序并标记建议保留（排序在模型层面，非数据层）
   ↓ 用户可手动点选覆盖（移动 ⭐ 标记）
确认 → 收集未保留项（非建议保留/非用户选中项）
   ↓
移到回收站（默认）| 移到文件夹 | 永久删除（双重确认）
   ↓
成功项从模型移除；失败项保留并提示
```

**不变量**：
- 组内任意时刻至少有一个保留候选（建议保留或用户选择），处理时绝不处理保留项
- 空白选择（用户取消全部选择）→ 回落到建议保留默认值后才允许批量操作
- 处理粒度按完整分组：只处理"该组未保留成员"，组与组互不影响

## 校验规则（取自规格 FR）

| 规则 | 来源 |
|---|---|
| width×height 为 0 的条目排组末 | FR-005/FR-006 + 宪法 IV |
| 批量操作仅作用于未保留项 | FR-010（宪法 III 强制） |
| 永久删除必须先勾选"我已确认"并二次确认 | FR-009 + 宪法 III |
| 失败项记录并提示，不中断 | FR-011 |
| 处理前展示文件清单 | FR-008/FR-009 |