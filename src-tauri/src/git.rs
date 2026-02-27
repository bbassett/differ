use git2::Repository;

use crate::types::{RefInfo, RefType};

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
}
