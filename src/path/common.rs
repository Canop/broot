use {
    std::path::{Components, PathBuf},
};

pub fn longest_common_ancestor(paths: &[PathBuf]) -> PathBuf {
    match paths.len() {
        0 => PathBuf::new(), // empty
        1 => paths[0].clone(),
        _ => {
            let cs0 = paths[0].components();
            let mut csi: Vec<Components> = paths
                .iter()
                .skip(1)
                .map(|p| p.components())
                .collect();
            let mut lca = PathBuf::new();
            for component in cs0 {
                for cs in &mut csi {
                    if cs.next() != Some(component) {
                        return lca;
                    }
                }
                lca.push(component);
            }
            lca
        }
    }
}
