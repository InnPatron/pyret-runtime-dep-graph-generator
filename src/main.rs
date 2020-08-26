use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use glob::{ GlobResult, glob };

mod graph;

use graph::*;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut args = env::args();
    args.next().unwrap();
    let output_path = args.next().unwrap();

    let mut graph = Graph::new();
    let omega_glob = args
        .fold(Box::new(std::iter::empty()) as Box<dyn Iterator<Item=GlobResult>>, |acc, mut p| {
            println!("Visiting: {}", p);
            p.push_str("*");
            let mini_glob = glob(&p).unwrap();
            Box::new(acc.chain(mini_glob))
        });

    for glob_res in omega_glob {
        let path = glob_res?;

        let string_path = path.to_str().expect("Input path not utf8").to_string();
        let (_, _, ext) =  get_data(&string_path);

        if !path.is_file() {
            continue
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        if ext == ".js" || ext == ".ts" || ext == ".arr.ts" || ext == ".arr.js" {
            let canon_path = canon_require_path(&string_path);
            generate_from_js(&mut graph, &canon_path, &mut reader)?;
        } else if ext == ".arr" {
            let canon_path = canon_require_path(&string_path);
            generate_from_pyret(&mut graph, &canon_path, &mut reader)?;
        } else if ext == ".arr.json" || ext == ".json" || ext.contains("swp") || ext.contains("stopped") {
            continue;
        } else {
            panic!("Unknown top-level extension: \"{}\" [{}]", ext, string_path);
        }

    }

    println!("Finished reading inputs...");
    println!("Writing results to {}", output_path);
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);
    writer.write("digraph mygraph {\n".as_bytes())?;
    graph.write_graph(&mut writer)?;
    writer.write("\n}".as_bytes())?;

    Ok(())
}

fn generate_from_pyret<T: BufRead>(graph: &mut Graph, current: &str, input: &mut T) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if let Some(to) = locate_dep(&line) {
            graph.add_edge(current, to);
        }
    }

    Ok(())
}

fn locate_dep(input: &str) -> Option<String> {
    if input.starts_with("include ") {
        let input = &input[8..];
        if !input.starts_with("from") {
            return Some(strip_protocol_dep(input));
        }
    }

    if input.starts_with("import ") {
        let input = &input[7..];
        let end = input.find(" as ").unwrap();
        let input = &input[..end];
        return Some(strip_protocol_dep(input));
    }

    None
}

fn strip_protocol_dep(input: &str) -> String {
    if let Some(file_index) = input.find("file(\"") {
        let input = &input[file_index + 6..];
        let end = input.find("\"").unwrap();
        let input = &input[..end];
        return canon_require_path(input);
    }

    if let Some(file_index) = input.find("jsfile(\"") {
        let input = &input[file_index + 8..];
        let end = input.find("\"").unwrap();
        let input = &input[..end];
        return canon_require_path(input);
    }

    input.to_string()
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

    let ( file_name, file_stem, ext ) = get_data(input);

    if ext == ".arr.js" || ext == ".arr"|| ext == ".arr.ts" {
        result.push_str(file_stem);
    } else if ext == ".js" ||  ext == ".ts" {
        result.push_str(file_name);
    } else if ext == "" {
        result.push_str(file_name);
    } else {
        panic!("Unable to handle extension: \"{}\" [{}]", ext, input);
    }


    result
}

fn get_data(input: &str) -> ( &str, &str, &str ) {
    let file_name_index = match input.rfind("/") {
        Some(i) => i + 1,
        None => 0
    };
    let file_name = &input[file_name_index..];
    let (file_stem, ext) = match file_name.find(".") {
        Some(first_dot_index) => {
            let ext = &file_name[first_dot_index..];
            let file_stem = &file_name[..first_dot_index];
            (file_stem, ext)
        }

        None => {
            (file_name, "")
        }
    };

    ( file_name, file_stem, ext )
}

fn locate_requires(input: &str) -> Option<String> {
    if let Some(require_index) = input.find("require(\"") {
        let start = require_index + 9;
        let target = &input[start..];
        let end = target.find("\"").unwrap();
        return Some((&target[..end]).to_owned());
    }

    None
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
        assert_eq!(canon_require_path("foo/bar.arr"), "bar");
    }

    #[test]
    fn test_locate_dep() {
        assert_eq!(locate_dep("include from global"), None);
        assert_eq!(locate_dep("include global"), Some("global".to_string()));
        assert_eq!(locate_dep("include file(\"foo/global.arr\")"), Some("global".to_string()));
        assert_eq!(locate_dep("import file(\"foo/global.arr\") as G"), Some("global".to_string()));
        assert_eq!(locate_dep("import global as G"), Some("global".to_string()));
    }

}
