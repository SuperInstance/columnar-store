# Columnar Store — Analytical Column-Oriented Storage Engine

**A columnar store** is a storage engine that organizes data by column rather than by row. Instead of storing records contiguously (`[row1, row2, row3]`), it stores each field contiguously (`[col1_values], [col2_values], [col3_values]`). This dramatically improves analytical query performance because queries that aggregate a few columns skip reading irrelevant data entirely.

## Why It Matters

Columnar storage is the engine behind every analytical database: Apache Parquet, ClickHouse, Amazon Redshift, Google BigQuery, Apache Druid, and DuckDB all use columnar layouts. The key insight is that analytical queries (OLAP) typically touch a small number of columns but a large number of rows. A row store reading `SELECT AVG(price) FROM orders` loads all columns into memory; a column store reads only the `price` column. For a table with 50 columns, that's a 50× I/O reduction. Combined with per-column compression (run-length encoding, dictionary encoding, delta encoding), columnar stores achieve 10–100× throughput on analytical workloads compared to row stores.

## How It Works

### Row vs. Column Layout

```
Row-Oriented:                  Column-Oriented:
┌──────┬───────┬────────┐     ┌──────┐ ┌───────┐ ┌────────┐
│ id=1 │ name=A │ val=10 │     │ id:  │ │ name: │ │ val:   │
│ id=2 │ name=B │ val=20 │     │ 1,2,3│ │ A,B,C │ │10,20,30│
│ id=3 │ name=C │ val=30 │     └──────┘ └───────┘ └────────┘
└──────┴───────┴────────┘
```

### Implementation

This crate stores each column as a `Vec<Box<dyn Any>>`, allowing heterogeneous types per column. Columns are registered with `add_column::<T>()` and rows are inserted as `HashMap<String, Box<dyn Any>>`:

```
ColumnarStore
├── "id"   → Vec<i64>   [1, 2, 3, ...]
├── "name" → Vec<String> ["alice", "bob", ...]
└── "age"  → Vec<i64>   [30, 25, 41, ...]
```

### Performance Characteristics

| Operation | Row Store | Column Store |
|---|---|---|
| Scan one column, N rows | `O(N × W)` where W = row width | `O(N)` — only that column is read |
| Point lookup by row ID | `O(1)` | `O(1)` — but may require scattered reads |
| Insert single row | `O(1)` append | `O(W)` — touches W column vectors |
| Query: `SUM(col)` | Reads all columns | Reads only `col` |

Column stores excel at **scan-heavy, column-selective** queries. They trade off write performance (small inserts touch multiple disjoint memory regions) for read performance on wide tables.

### Compression

Each column's homogeneous type enables type-specific compression:
- **Run-length encoding (RLE)**: consecutive identical values → `(value, count)`.
- **Dictionary encoding**: low-cardinality strings → integer indices.
- **Delta encoding**: sorted values → differences from previous.

Typical compression ratios: 3–10× on real-world datasets.

## Quick Start

```rust
use std::collections::HashMap;

let mut store = ColumnarStore::new();
store.add_column::<i64>("id");
store.add_column::<String>("name");

let mut row = HashMap::new();
row.insert("id".to_string(), Box::new(42i64));
row.insert("name".to_string(), Box::new("fleet".to_string()));
store.insert(row);

println!("Rows: {}", store.row_count());
println!("Columns: {:?}", store.column_names());
```

## API

| Method | Description |
|---|---|
| `ColumnarStore::new()` | Create an empty store. |
| `add_column::<T>(name)` | Register a typed column. `O(1)`. |
| `insert(row)` | Append values to each column. `O(W)` where W = columns. |
| `row_count()` | Total rows. `O(1)`. |
| `column_names()` | Names of all registered columns. |

## Architecture Notes

The columnar store is a storage primitive on the γ (generation/storage) side of γ + η = C in SuperInstance. It provides the analytical scan performance that metric aggregation and fleet telemetry queries require, complementing the row-oriented stores used for transactional workloads. See [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Stonebraker, M. et al. (2005). *C-Store: A Column-oriented DBMS*. VLDB. — The seminal columnar database paper.
2. Abadi, D., Madden, S., & Ferreira, M. (2006). *Integrating Compression and Execution in Column-Oriented Database Systems*. SIGMOD. — Per-column compression strategies.
3. Parquet Format Specification. <https://parquet.apache.org/docs/file-format/>

## License

MIT
