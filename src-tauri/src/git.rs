use git2::Repository;

use crate::types::{DiffFile, DiffHunk, DiffLine, DiffResult, FileStatus, LineType, RefInfo, RefType};

pub fn discover_repo(path: &str) -> Result<Repository, String> {
    Repository::discover(path).map_err(|e| format!("Failed to discover repo: {}", e))
}

pub fn list_refs(repo: &Repository) -> Result<Vec<RefInfo>, String> {
    let mut refs = Vec::new();

    // Local branches
    let branches = repo
        .branches(Some(git2::BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    for branch in branches {
        let (branch, _) = branch.map_err(|e| format!("Failed to read branch: {}", e))?;
        if let Some(name) = branch
            .name()
            .map_err(|e| format!("Invalid branch name: {}", e))?
        {
            refs.push(RefInfo {
                name: name.to_string(),
                ref_type: RefType::Branch,
            });
        }
    }

    // Tags
    let tag_names = repo
        .tag_names(None)
        .map_err(|e| format!("Failed to list tags: {}", e))?;

    for tag_name in tag_names.iter().flatten() {
        refs.push(RefInfo {
            name: tag_name.to_string(),
            ref_type: RefType::Tag,
        });
    }

    Ok(refs)
}

pub fn generate_diff(repo: &Repository, base: &str, compare: &str) -> Result<DiffResult, String> {
    let base_ref = format!("refs/heads/{}", base);
    let compare_ref = format!("refs/heads/{}", compare);

    let base_obj = repo
        .revparse_single(&base_ref)
        .map_err(|e| format!("Failed to resolve '{}': {}", base, e))?;
    let compare_obj = repo
        .revparse_single(&compare_ref)
        .map_err(|e| format!("Failed to resolve '{}': {}", compare, e))?;

    let base_tree = base_obj
        .peel_to_tree()
        .map_err(|e| format!("Failed to get tree for '{}': {}", base, e))?;
    let compare_tree = compare_obj
        .peel_to_tree()
        .map_err(|e| format!("Failed to get tree for '{}': {}", compare, e))?;

    let diff = repo
        .diff_tree_to_tree(Some(&base_tree), Some(&compare_tree), None)
        .map_err(|e| format!("Failed to generate diff: {}", e))?;

    let mut files = Vec::new();

    for idx in 0..diff.deltas().len() {
        let delta = diff.get_delta(idx).unwrap();

        let status = match delta.status() {
            git2::Delta::Added => FileStatus::Added,
            git2::Delta::Deleted => FileStatus::Deleted,
            git2::Delta::Modified => FileStatus::Modified,
            git2::Delta::Renamed => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };

        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let old_path = if matches!(status, FileStatus::Renamed) {
            delta
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let mut hunks = Vec::new();

        if let Ok(patch) = git2::Patch::from_diff(&diff, idx) {
            if let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk, _count) = patch.hunk(hunk_idx).unwrap();
                    let mut lines = Vec::new();

                    for line_idx in 0..patch.num_lines_in_hunk(hunk_idx).unwrap_or(0) {
                        if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
                            let line_type = match line.origin() {
                                '+' => LineType::Add,
                                '-' => LineType::Delete,
                                _ => LineType::Context,
                            };

                            let content = std::str::from_utf8(line.content())
                                .unwrap_or("")
                                .trim_end_matches('\n')
                                .to_string();

                            lines.push(DiffLine {
                                line_type,
                                content,
                                old_num: line.old_lineno(),
                                new_num: line.new_lineno(),
                            });
                        }
                    }

                    hunks.push(DiffHunk {
                        old_start: hunk.old_start(),
                        old_lines: hunk.old_lines(),
                        new_start: hunk.new_start(),
                        new_lines: hunk.new_lines(),
                        lines,
                    });
                }
            }
        }

        files.push(DiffFile {
            path,
            status,
            old_path,
            hunks,
        });
    }

    Ok(DiffResult {
        base_ref: base.to_string(),
        compare_ref: compare.to_string(),
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup_test_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();

        // Create an initial commit so branches exist
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            let test_file = path.join("test.txt");
            fs::write(&test_file, "hello").unwrap();
            index.add_path(Path::new("test.txt")).unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }

        repo
    }

    #[test]
    fn test_discover_repo() {
        let dir = tempfile::tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let repo = discover_repo(dir.path().to_str().unwrap());
        assert!(repo.is_ok());
    }

    #[test]
    fn test_discover_nonexistent() {
        let result = discover_repo("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_branches() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        // Create another branch
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();

        let refs = list_refs(&repo).unwrap();
        let branch_names: Vec<&str> = refs
            .iter()
            .filter(|r| matches!(r.ref_type, RefType::Branch))
            .map(|r| r.name.as_str())
            .collect();

        assert!(branch_names.contains(&"main") || branch_names.contains(&"master"));
        assert!(branch_names.contains(&"feature"));
    }

    #[test]
    fn test_list_tags() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v1.0", head.as_object(), false)
            .unwrap();

        let refs = list_refs(&repo).unwrap();
        let tag_names: Vec<&str> = refs
            .iter()
            .filter(|r| matches!(r.ref_type, RefType::Tag))
            .map(|r| r.name.as_str())
            .collect();

        assert!(tag_names.contains(&"v1.0"));
    }

    fn commit_on_branch(repo: &Repository, path: &std::path::Path, branch_name: &str) {
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch(branch_name, &head, false).unwrap();
        repo.set_head(&format!("refs/heads/{}", branch_name))
            .unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();
        let _ = (repo, path); // keep in scope
    }

    fn make_commit(repo: &Repository, _path: &std::path::Path, msg: &str) {
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_generate_diff_modified_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        commit_on_branch(&repo, dir.path(), "feature");

        fs::write(dir.path().join("test.txt"), "hello\nworld\n").unwrap();
        make_commit(&repo, dir.path(), "modify");

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        assert_eq!(diff.base_ref, "main");
        assert_eq!(diff.compare_ref, "feature");
        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "test.txt");
        assert!(matches!(diff.files[0].status, FileStatus::Modified));
        assert!(!diff.files[0].hunks.is_empty());
    }

    #[test]
    fn test_generate_diff_added_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        commit_on_branch(&repo, dir.path(), "feature");

        fs::write(dir.path().join("new.txt"), "new file\n").unwrap();
        make_commit(&repo, dir.path(), "add file");

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        let new_file = diff.files.iter().find(|f| f.path == "new.txt").unwrap();
        assert!(matches!(new_file.status, FileStatus::Added));
    }

    #[test]
    fn test_generate_diff_deleted_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        commit_on_branch(&repo, dir.path(), "feature");

        fs::remove_file(dir.path().join("test.txt")).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index.remove_path(Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        {
            let tree = repo.find_tree(tree_id).unwrap();
            let parent = repo.head().unwrap().peel_to_commit().unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "delete file", &tree, &[&parent])
                .unwrap();
        }

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        let deleted = diff.files.iter().find(|f| f.path == "test.txt").unwrap();
        assert!(matches!(deleted.status, FileStatus::Deleted));
    }
}
