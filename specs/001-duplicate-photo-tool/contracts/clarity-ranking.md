# Contract: 清晰度评分 API（czkawka_core 新增）

**Phase 1 输出** | **2026-08-17**

## 定位

纯函数模块，供 Krokiet/CLI 结果展示层对相似组内成员做清晰度排序与建议保留判定。
**不接触磁盘、不依赖 GUI、无状态**，可单元测试、可并行调用。

## 函数契约

### `clarity_score(entry) -> u64`

```rust
pub fn clarity_score(entry: &ImagesEntry) -> u64
```

- 输入：`ImagesEntry` 引用（只读）
- 输出：清晰度得分 `(width as u64) * (height as u64) * 1000 + (size / 1024)`
- 约束：
  - `width == 0 || height == 0` → 返回 `0`（解码失败条目，绝不被建议保留）
  - 无 panics；接受任意合法 ImagesEntry

### `rank_group_by_clarity(group: &mut [ImagesEntry]) -> usize`

```rust
pub fn rank_group_by_clarity(group: &mut [ImagesEntry]) -> usize
```

- 输入：可变引用的相似组成员切片
- 行为：按 `clarity_score` 降序原地排序；同分时按 `path` 字典序升序（稳定、确定性）
- 输出：建议保留索引（排序后得分最高者的下标，`0`；空组返回 `0` 且不变更）
- 约束：O(n log n)；不为任何成员修改其既有字段

### `group_keep_index(group: &[ImagesEntry]) -> Option<usize>`

```rust
pub fn group_keep_index(group: &[ImagesEntry]) -> Option<usize>
```

- 输入：只读组成员
- 输出：建议保留者的下标（`None` 当组为空或组内所有成员得分均为 0）
- 用途：GUI 层查询"该组默认应标记哪一行"，无需重新排序

## 不变量

1. 排序只影响展示顺序，不改变 `similar_vectors` 的组归属
2. 评分结果对同一输入唯一确定（无随机、无时间依赖）
3. 本模块不调用 `std::fs`、不调用 image 解码——性能零损耗

## 调用方约束

- Krokiet 在结果模型构建完成后、渲染前调用 `rank_group_by_clarity` 排序单组行数据
- 建议保留标记（⭐）仅表示"组内清晰度第一"，用户点选后可覆盖；覆盖只改展示标记，
  不改 `ImagesEntry` 数据
- 批量处理前必须重新读取当前模型中的保留标记（用户可能已覆盖），处理对象 = 未标记保留的成员

## 测试承诺（对照规格 FR-005/FR-006）

| 场景 | 预期 |
|---|---|
| 组内 [1000×800, 800×600, 500×400] | 排序后 1000×800 在最前，keep_index=0 |
| 组内 [800×600 大体积, 800×600 小体积] | 大体积在前 |
| 组内含 width=0 条目 | 0 尺寸条目排末，keep_index 指向首个非零得分者 |
| 组内全为 0 尺寸 | keep_index=None |
| 同分不同路径 | 按 path 字典序，结果可复现 |
| 空组 | keep_index=None，排序为 no-op |