// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use arrow_array::RecordBatch;
use arrow_schema::Schema;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use futures::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use lancedb::database::CreateTableMode;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use lancedb::{connect, Connection, Table};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub type SearchResult<T> = Result<T, Box<CommonError>>;

static DB: OnceCell<Connection> = OnceCell::const_new();

fn err(msg: impl Into<String>) -> Box<CommonError> {
    Box::new(CommonError::CommonError(msg.into()))
}

fn err_from(e: impl std::fmt::Display) -> Box<CommonError> {
    err(e.to_string())
}

fn lancedb_path() -> String {
    let conf = broker_config();
    format!("{}/_lancedb", conf.data_path)
}

pub async fn init() -> SearchResult<()> {
    let path = lancedb_path();
    let conn = connect(&path).execute().await.map_err(err_from)?;
    DB.set(conn).map_err(|_| err("lancedb already initialized"))
}

fn get_conn() -> SearchResult<&'static Connection> {
    DB.get()
        .ok_or_else(|| err("lancedb not initialized, call lancedb::init() first"))
}

pub async fn create_table(name: &str, schema: Arc<Schema>) -> SearchResult<Table> {
    get_conn()?
        .create_empty_table(name, schema)
        .mode(CreateTableMode::Overwrite)
        .execute()
        .await
        .map_err(err_from)
}

pub async fn open_table(name: &str) -> SearchResult<Table> {
    get_conn()?
        .open_table(name)
        .execute()
        .await
        .map_err(err_from)
}

pub async fn insert(table: &Table, batch: RecordBatch) -> SearchResult<()> {
    table
        .add(vec![batch])
        .execute()
        .await
        .map(|_| ())
        .map_err(err_from)
}

/// `updates`: list of (column_name, sql_expression) pairs.
pub async fn update(table: &Table, filter: &str, updates: Vec<(&str, &str)>) -> SearchResult<()> {
    let mut builder = table.update().only_if(filter);
    for (col, expr) in updates {
        builder = builder.column(col, expr);
    }
    builder.execute().await.map(|_| ()).map_err(err_from)
}

pub async fn delete(table: &Table, filter: &str) -> SearchResult<()> {
    table.delete(filter).await.map(|_| ()).map_err(err_from)
}

pub async fn vector_search(
    table: &Table,
    vector: Vec<f32>,
    limit: usize,
    offset: usize,
    columns: Option<&[&str]>,
    filter: Option<&str>,
) -> SearchResult<Vec<RecordBatch>> {
    let mut q = table.vector_search(vector).map_err(err_from)?.limit(limit);

    if offset > 0 {
        q = q.offset(offset);
    }
    if let Some(cols) = columns {
        q = q.select(Select::columns(cols));
    }
    if let Some(f) = filter {
        q = q.only_if(f);
    }

    q.execute()
        .await
        .map_err(err_from)?
        .try_collect::<Vec<_>>()
        .await
        .map_err(err_from)
}

pub async fn full_text_search(
    table: &Table,
    query: &str,
    limit: usize,
    offset: usize,
    columns: Option<&[&str]>,
    filter: Option<&str>,
) -> SearchResult<Vec<RecordBatch>> {
    let mut q = table
        .query()
        .full_text_search(FullTextSearchQuery::new(query.to_string()))
        .limit(limit);

    if offset > 0 {
        q = q.offset(offset);
    }
    if let Some(cols) = columns {
        q = q.select(Select::columns(cols));
    }
    if let Some(f) = filter {
        q = q.only_if(f);
    }

    match q.execute().await {
        Ok(stream) => stream.try_collect::<Vec<_>>().await.map_err(err_from),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("INVERTED index") || msg.contains("full text search") {
                Ok(vec![])
            } else {
                Err(err_from(e))
            }
        }
    }
}

pub async fn create_fts_index(table: &Table, columns: &[&str]) -> SearchResult<()> {
    table
        .create_index(columns, Index::FTS(Default::default()))
        .replace(true)
        .execute()
        .await
        .map_err(err_from)
}

pub async fn create_vector_index(table: &Table, column: &str) -> SearchResult<()> {
    use lancedb::index::vector::IvfHnswSqIndexBuilder;
    table
        .create_index(
            &[column],
            Index::IvfHnswSq(IvfHnswSqIndexBuilder::default()),
        )
        .replace(true)
        .execute()
        .await
        .map_err(err_from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{FixedSizeListArray, Int64Array, StringArray};
    use arrow_schema::{DataType, Field};
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use lance_arrow::FixedSizeListArrayExt;

    fn make_schema(dim: i32) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dim),
                false,
            ),
        ]))
    }

    fn make_batch(schema: Arc<Schema>, dim: i32) -> RecordBatch {
        let ids = Arc::new(Int64Array::from(vec![1i64, 2, 3]));
        let texts = Arc::new(StringArray::from(vec![
            "RobustMQ is a cloud-native message queue",
            "LanceDB is a vector database",
            "Rust is a systems programming language",
        ]));
        let flat: Vec<f32> = (0..3 * dim as usize).map(|i| i as f32 * 0.01).collect();
        let values: arrow_array::ArrayRef = Arc::new(arrow_array::Float32Array::from(flat));
        let vectors = Arc::new(FixedSizeListArray::try_new_from_values(values, dim).unwrap());
        RecordBatch::try_new(schema, vec![ids, texts, vectors]).unwrap()
    }

    #[tokio::test]
    async fn test_basic_operations() {
        init_broker_conf_by_config(BrokerConfig::default());
        init().await.unwrap();

        let dim = 4i32;
        let schema = make_schema(dim);
        let table = create_table("docs", schema.clone()).await.unwrap();

        insert(&table, make_batch(schema, dim)).await.unwrap();

        let results = vector_search(
            &table,
            vec![0.0f32; dim as usize],
            3,
            0,
            Some(&["id", "text"]),
            None,
        )
        .await
        .unwrap();
        assert!(!results.is_empty());

        create_fts_index(&table, &["text"]).await.unwrap();
        let fts = full_text_search(&table, "message queue", 3, 0, Some(&["id", "text"]), None)
            .await
            .unwrap();
        assert!(!fts.is_empty());

        update(&table, "id = 1", vec![("text", "'updated text'")])
            .await
            .unwrap();
        delete(&table, "id = 3").await.unwrap();
    }
}
