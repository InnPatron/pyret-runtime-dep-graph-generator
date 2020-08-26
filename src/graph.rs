use std::io::{Write};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::borrow::Borrow;

pub struct Graph {
    graph: HashMap<String, Vec<String>>
}

impl Graph {

    pub fn new() -> Self {
        Graph { graph: HashMap::new() }
    }

    pub fn add_edge<T1: Into<String>, T2: Borrow<String> + Into<String> + Clone>(&mut self, from: T1, to: T2) {
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

    pub fn write_graph<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        for (from, tos) in self.graph.iter() {
            for to in tos.iter() {
                writer.write("\"".as_bytes())?;
                writer.write(from.as_bytes())?;
                writer.write("\"".as_bytes())?;
                writer.write(" -> ".as_bytes())?;
                writer.write("\"".as_bytes())?;
                writer.write(to.as_bytes())?;
                writer.write("\"".as_bytes())?;
                writer.write(";\n".as_bytes())?;
            }
        }

        Ok(())
    }
}
