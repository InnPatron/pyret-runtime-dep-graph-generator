use std::env;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::path::PathBuf;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() {
    let args: Vec<PathBuf> = env::args()
        .skip(1)
        .map(|p| PathBuf::from(p))
        .collect();

    println!("{:?}", args);
}

fn generate_from_js<T: BufRead>(graph: &mut Graph, current: &str, input: &mut T) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if let Some(to) = locate_requires(&line) {
            let to = canon_require_path(&to);
            graph.add_edge(current, to);
        }
    }

    Ok(())
}

fn canon_require_path(input: &str) -> String {
    let mut result = String::new();

    let file_name_index = match input.rfind("/") {
        Some(i) => i + 1,
        None => 0
    };
    let file_name = &input[file_name_index..];
    let first_dot_index = file_name.find(".").expect("No file extension");
    let ext = &file_name[first_dot_index..];
    let file_stem = &file_name[..first_dot_index];

    dbg!(file_name);
    dbg!(file_stem);
    dbg!(ext);
    if ext == ".arr.js" {
        result.push_str(file_stem);
    } else if ext == ".js" ||  ext == ".ts" {
        result.push_str(file_name);
    } else {
        panic!("Unable to handle extension for: {}", input);
    }


    result
}

fn locate_requires(input: &str) -> Option<String> {
    if let Some(require_index) = input.find("require(\"") {
        let start = require_index + 9;
        let target = &input[start..];
        let end = input.find("\"").unwrap() - 1;
        return Some((&target[..end]).to_owned());
    }

    None
}

struct Graph {
    graph: HashMap<String, Vec<String>>
}

impl Graph {
    fn add_edge<T1: Into<String>, T2: Borrow<String> + Into<String> + Clone>(&mut self, from: T1, to: T2) {
        if self.graph.contains_key(to.borrow()) == false {
            self.graph.insert(to.clone().into(), Vec::new());
        }

        match self.graph.entry(from.into()) {
            Entry::Occupied(o) => {
                o.into_mut().push(to.into());
            }
            Entry::Vacant(e) => {
                e.insert(vec![to.into()]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_locate_requires() {
        assert_eq!(locate_requires("require(\"foo/bar\")"), Some("foo/bar".to_string()));
    }

    #[test]
    fn test_canon_path() {
        assert_eq!(canon_require_path("foo/bar.arr.js"), "bar");
        assert_eq!(canon_require_path("foo/bar.js"), "bar.js");
        assert_eq!(canon_require_path("foo/bar.ts"), "bar.ts");
    }

}
