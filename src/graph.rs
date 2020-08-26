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
}
