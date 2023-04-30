use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

fn main() {
    let dir = Path::new("tests");
    let mut files = vec![];
    recurse_schemas(dir.read_dir().unwrap(), &mut files);
    for file in files.iter() {
        let parents = file
            .parent()
            .unwrap()
            .strip_prefix(dir)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        capnpc::CompilerCommand::new()
            .file(file)
            .output_path(".")
            .default_parent_module(parents)
            .run()
            .expect("compiling schema");
    }
}

fn recurse_schemas(dir: ReadDir, schemas: &mut Vec<PathBuf>) {
    for path in dir
        .into_iter()
        .filter_map(|entry| entry.map(|entry| entry.path()).ok())
    {
        if path.is_dir() {
            recurse_schemas(path.read_dir().unwrap(), schemas);
        } else if path.extension().unwrap_or_default() == "capnp" {
            schemas.push(path);
        }
    }
}
