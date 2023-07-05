use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Tree {
    File,
    Directory(HashMap<String, Tree>),
}

impl Tree {
    pub fn map_mut(&mut self) -> Option<&mut HashMap<String, Tree>> {
        match self {
            Tree::File => None,
            Tree::Directory(map) => Some(map),
        }
    }

    pub fn build_program(self) -> String {
        let mut result = "mod main_module {\n".to_string();

        match self {
            Tree::File => unreachable!(),
            Tree::Directory(map) => {
                for (name, tree) in map {
                    match tree {
                        Tree::File => {
                            result.push_str(&format!(
                                "mod {} {{\n",
                                name.strip_suffix(".ac").unwrap()
                            ));
                            // result
                            // .push_str(&format!("{}\n", std::fs::read_to_string(name).unwrap()));

                            result.push_str("}\n");
                        }
                        tree => {
                            result.push_str(&format!("mod {} {{\n", name));
                            result.push_str(&tree.build_program());
                            result.push_str("}\n");
                        }
                    }
                }
            }
        }

        result.push_str("}\n");

        result
    }
}

#[derive(Debug)]
pub struct FileTree {
    extension: String,
    tree: Tree,
    project_directory: PathBuf,
}

impl FileTree {
    pub fn new(project_directory: PathBuf) -> Self {
        FileTree {
            extension: String::from("ac"),
            tree: Tree::Directory(HashMap::new()),
            project_directory,
        }
    }

    pub fn insert(&mut self, path: &Path, prefix: &Path) {
        let mut map = self.tree.map_mut().unwrap();

        let mut buff = OsStr::new("");
        for str in path.strip_prefix(prefix).unwrap() {
            let dir = buff;
            buff = str;

            map = map
                .entry(dir.to_str().unwrap().to_string())
                .or_insert(Tree::Directory(HashMap::new()))
                .map_mut()
                .unwrap();
        }

        map.insert(buff.to_str().unwrap().to_string(), Tree::File);
    }

    pub fn build_program(self) -> String {
        self.tree.build_program()
    }
}
