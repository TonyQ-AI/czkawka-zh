# Engine Review Notes: Similar Images 扫描与结果流

**2026-08-17** | US1 T015 输出

## 结论概览

上游 czkawka 的相似图片扫描与分组引擎成熟可用，US1（扫描分组）无需改核心代码，
仅需将结果排序从"汉明距离+大小"替换为"清晰度"即可实现 US2。

## 结果流（已读源码确认）

1. **核心引擎**：`czkawka_core/src/tools/similar_images/core.rs`
   - `SimilarImages::new(params)` 构建工具（mod.rs:27）
   - `check_for_similar_images` 遍历目录（按 inode 分组，隐藏硬链接可选）
   - 哈希阶段填充 `file_entry.width = dimensions.0; height = dimensions.1`（core.rs:187-188）
   - 分组结果存入 `similar_vectors: Vec<Vec<ImagesEntry>>` 和
     `similar_referenced_vectors: Vec<(ImagesEntry, Vec<ImagesEntry>)>`（core.rs:523）

2. **Krokiet 扫描回调**：`krokiet/src/connect_scan/similar_images.rs`
   - `scan_similar_images`（第 19 行）启动后台线程执行 `tool.search()`
   - 结果形态：`(Option<ImagesEntry> 参考项, Vec<ImagesEntry> 成员)`（第 49-57 行）
   - **排序插入点（第 59-62 行）**：
     ```rust
     for (_first_entry, vec_fe) in &mut vector {
         vec_fe.par_sort_unstable_by_key(|e| (e.difference, u64::MAX - e.size));
     }
     vector.sort_by_key(|(_header, vc)| u64::MAX - vc.iter().map(|e| e.size).sum::<u64>());
     ```
     当前按"相似度差异 + 大小倒序"排成员；`rank_group_by_clarity` 应在此替换为
     "清晰度降序"（保留参考项在原位，成员内重排）。

3. **模型转换**：`prepare_data_model_similar_images`（第 121-146 行）
   - `val_str` 6 列：相似度文本、大小、分辨率串、文件名、路径、修改时间
   - `val_int` 8 列：修改时间高低位、大小高低位、宽度、高度、像素数、差异值
   - `width/height/pixel_count` 已在列中——清晰度可视化不需要新增数据列

4. **GUI 渲染**：`krokiet/ui/screens/main_lists.slint`（第 123-135 行）
   - `SelectableTableView` 通用组件；列定义在 `Settings.similar_images_column_name`
     （settings.slint:304：Selection, Similarity, Size, Dimensions, File Name, Path, Modification Date）
   - `SingleMainListModel`（globals/common.slint:51）：checked/header_row/val_str/val_int
   - 勾选框在列 0；**"建议保留"标记最佳载体为新增一个标记列**或改动 Name 列前缀
   - `similar_images_data_idx = [5, 4, -1, -1]`（gui_state.slint:114）指向 Path/Name 列索引

## 对 US2 实现的影响

- 排序替换点已锁定：`connect_scan/similar_images.rs:59-62` 用 `rank_group_by_clarity`
- 建议保留标记：决定新增 "Keep" 列（中英翻译跟进）并在行数据中打标，或复用 Name 列加 ⭐
  前缀——实现时二选一，优先新增列（不污染既有列数据）

## 未改动项

- `czkawka_core` 扫描/哈希/分组算法：零改动（上游成熟）
- 检查项：默认相似度预设为 High（阈值 15@8bit），不排除同大小/同分辨率——默认即可发现
  "异名同内容"照片（US1 断言 1 满足）