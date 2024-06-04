use std::path::PathBuf;

pub fn paths_to_cmp_string(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<String>>()
        .join(",")
}
