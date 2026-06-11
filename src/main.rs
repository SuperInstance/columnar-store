use std::collections::HashMap;

/// Columnar storage engine optimized for analytical queries
struct ColumnarStore {
    columns: HashMap<String, Vec<Box<dyn std::any::Any>>>,
    row_count: usize,
}

impl ColumnarStore {
    fn new() -> Self {
        Self { columns: HashMap::new(), row_count: 0 }
    }

    fn add_column<T: 'static + Clone>(&mut self, name: &str) {
        self.columns.insert(name.to_string(), vec![Box::new(Vec::<T>::new())]);
    }

    fn insert(&mut self, row: HashMap<String, Box<dyn std::any::Any>>) {
        for (col, val) in row {
            if let Some(column) = self.columns.get_mut(&col) {
                column.push(val);
            }
        }
        self.row_count += 1;
    }

    fn row_count(&self) -> usize { self.row_count }

    fn column_names(&self) -> Vec<&String> {
        self.columns.keys().collect()
    }
}

fn main() {
    let mut store = ColumnarStore::new();
    store.add_column::<i64>("id");
    store.add_column::<String>("name");
    println!("Columnar store initialized with columns: {:?}", store.column_names());
    println!("Row count: {}", store.row_count());
}
