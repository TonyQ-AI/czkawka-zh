use crate::tools::similar_images::ImagesEntry;

/// Clarity score of one image entry.
/// Resolution (pixels) wins, then file size in KiB.
/// Zero dimensions (undecodable) score 0 and never get recommended for keeping.
pub fn clarity_score(entry: &ImagesEntry) -> u64 {
    let pixels = (entry.width as u64).saturating_mul(entry.height as u64);
    if pixels == 0 {
        return 0;
    }
    pixels * 1000 + entry.size / 1024
}

/// Sort a group in place by clarity score, descending.
/// Ties break by path (lexicographic, ascending) for determinism.
/// Returns the index of the recommended keeper (0 for non-empty groups).
pub fn rank_group_by_clarity(group: &mut [ImagesEntry]) -> usize {
    group.sort_by(|a, b| clarity_score(b).cmp(&clarity_score(a)).then_with(|| a.path.cmp(&b.path)));
    0
}

/// Index of the recommended keeper in a group, or None when the group is
/// empty or every member has zero dimensions.
pub fn group_keep_index(group: &[ImagesEntry]) -> Option<usize> {
    group.iter().position(|e| clarity_score(e) > 0)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn entry(width: u32, height: u32, size: u64, path: &str) -> ImagesEntry {
        ImagesEntry {
            path: PathBuf::from(path),
            size,
            width,
            height,
            modified_date: 0,
            hashes: Vec::new(),
            difference: 0,
        }
    }

    #[test]
    fn higher_resolution_ranks_first() {
        let mut group = vec![
            entry(500, 400, 200_000, "photos/small.jpg"),
            entry(1000, 800, 100_000, "photos/big.jpg"),
            entry(800, 600, 150_000, "photos/mid.jpg"),
        ];
        rank_group_by_clarity(&mut group);
        assert_eq!(group[0].width, 1000);
        assert_eq!(group[0].height, 800);
        assert_eq!(group[1].width, 800);
        assert_eq!(group[2].width, 500);
    }

    #[test]
    fn same_resolution_prefers_larger_file() {
        let mut group = vec![entry(800, 600, 100_000, "photos/a.jpg"), entry(800, 600, 300_000, "photos/b.jpg")];
        rank_group_by_clarity(&mut group);
        assert_eq!(group[0].size, 300_000);
    }

    #[test]
    fn zero_dimensions_rank_last() {
        let mut group = vec![entry(0, 0, 50_000, "photos/broken.png"), entry(800, 600, 120_000, "photos/good.jpg")];
        rank_group_by_clarity(&mut group);
        assert_eq!(group[0].path.to_string_lossy(), "photos/good.jpg");
        assert_eq!(group[1].path.to_string_lossy(), "photos/broken.png");
    }

    #[test]
    fn keep_index_skips_zero_dimensions() {
        let group = vec![entry(0, 0, 50_000, "photos/broken.png"), entry(800, 600, 120_000, "photos/good.jpg")];
        let keep = group_keep_index(&group);
        assert_eq!(keep, Some(1));
    }

    #[test]
    fn keep_index_none_when_all_zero_dimensions() {
        let group = vec![entry(0, 0, 50_000, "photos/a.png"), entry(0, 0, 80_000, "photos/b.png")];
        assert_eq!(group_keep_index(&group), None);
    }

    #[test]
    fn keep_index_none_for_empty_group() {
        let group: Vec<ImagesEntry> = Vec::new();
        assert_eq!(group_keep_index(&group), None);
    }

    #[test]
    fn tie_breaks_by_path_deterministically() {
        let mut group = vec![entry(800, 600, 200_000, "photos/b.jpg"), entry(800, 600, 200_000, "photos/a.jpg")];
        rank_group_by_clarity(&mut group);
        assert_eq!(group[0].path.to_string_lossy(), "photos/a.jpg");
        assert_eq!(group[1].path.to_string_lossy(), "photos/b.jpg");
    }

    #[test]
    fn clarity_score_zero_for_zero_dimensions() {
        assert_eq!(clarity_score(&entry(0, 0, 99_999, "photos/x.png")), 0);
        assert_eq!(clarity_score(&entry(100, 0, 99_999, "photos/y.png")), 0);
    }
}
