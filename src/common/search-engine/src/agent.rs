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

use crate::lancedb::{
    create_fts_index, create_table, create_vector_index, delete, full_text_search, insert,
    open_table, vector_search, SearchResult,
};
use a2a_types::AgentCard;
use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use common_base::error::common::CommonError;
use lance_arrow::FixedSizeListArrayExt;
use std::sync::Arc;

const TABLE_NAME: &str = "mq9_agents";
const VECTOR_DIM: i32 = 384;

fn err(msg: impl Into<String>) -> Box<CommonError> {
    Box::new(CommonError::CommonError(msg.into()))
}

fn agent_id(tenant: &str, name: &str) -> String {
    format!("{}\x00{}", tenant, name)
}

fn build_skill_summary(card: &AgentCard) -> String {
    card.skills
        .iter()
        .map(|s| {
            let tags = s.tags.join(" ");
            format!("{} {} {}", s.name, s.description, tags)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn make_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("agent_id", DataType::Utf8, false),
        Field::new("tenant", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("description", DataType::Utf8, false),
        Field::new("search_text", DataType::Utf8, false),
        Field::new("agent_info", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                VECTOR_DIM,
            ),
            false,
        ),
    ]))
}

fn make_batch(
    tenant: &str,
    card: &AgentCard,
    agent_info_json: &str,
    vector: Vec<f32>,
) -> SearchResult<RecordBatch> {
    let schema = make_schema();
    let id = agent_id(tenant, &card.name);
    let search_text = format!(
        "{} {} {}",
        card.name,
        card.description,
        build_skill_summary(card)
    );

    let values: arrow_array::ArrayRef = Arc::new(arrow_array::Float32Array::from(vector));
    let vectors: arrow_array::ArrayRef = Arc::new(
        FixedSizeListArray::try_new_from_values(values, VECTOR_DIM)
            .map_err(|e| err(format!("build vector array failed: {e}")))?,
    );

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![id.as_str()])),
            Arc::new(StringArray::from(vec![tenant])),
            Arc::new(StringArray::from(vec![card.name.as_str()])),
            Arc::new(StringArray::from(vec![card.description.as_str()])),
            Arc::new(StringArray::from(vec![search_text.as_str()])),
            Arc::new(StringArray::from(vec![agent_info_json])),
            vectors,
        ],
    )
    .map_err(|e| err(format!("build record batch failed: {e}")))
}

pub async fn register_agent(
    tenant: &str,
    card: &AgentCard,
    agent_info_json: &str,
    vector: Vec<f32>,
) -> SearchResult<()> {
    let schema = make_schema();
    let table = match open_table(TABLE_NAME).await {
        Ok(t) => {
            let current = t.schema().await.map_err(|e| err(e.to_string()))?;
            if current.fields() != schema.fields() {
                create_table(TABLE_NAME, schema).await?
            } else {
                t
            }
        }
        Err(_) => create_table(TABLE_NAME, schema).await?,
    };

    let filter = format!("agent_id = '{}'", agent_id(tenant, &card.name));
    delete(&table, &filter).await?;

    let batch = make_batch(tenant, card, agent_info_json, vector)?;
    insert(&table, batch).await?;

    create_fts_index(&table, &["search_text"]).await?;
    create_vector_index(&table, "vector").await
}

pub async fn unregister_agent(tenant: &str, name: &str) -> SearchResult<()> {
    let table = open_table(TABLE_NAME).await?;
    let filter = format!("agent_id = '{}'", agent_id(tenant, name));
    delete(&table, &filter).await
}

pub async fn search_agents_by_vector(
    vector: Vec<f32>,
    limit: usize,
    offset: usize,
    tenant: Option<&str>,
) -> SearchResult<Vec<AgentSearchResult>> {
    let table = open_table(TABLE_NAME).await?;
    let filter = tenant.map(|t| format!("tenant = '{}'", t));
    let batches = vector_search(
        &table,
        vector,
        limit,
        offset,
        Some(&["agent_id", "name", "description", "agent_info"]),
        filter.as_deref(),
    )
    .await?;
    Ok(extract_results(batches))
}

pub async fn search_agents_by_text(
    query: &str,
    limit: usize,
    offset: usize,
    tenant: Option<&str>,
) -> SearchResult<Vec<AgentSearchResult>> {
    let table = open_table(TABLE_NAME).await?;
    let filter = tenant.map(|t| format!("tenant = '{}'", t));
    let batches = full_text_search(
        &table,
        query,
        limit,
        offset,
        Some(&["agent_id", "name", "description", "agent_info"]),
        filter.as_deref(),
    )
    .await?;
    Ok(extract_results(batches))
}

pub fn embed_text(card: &AgentCard) -> String {
    let mut parts = vec![card.name.clone(), card.description.clone()];
    for skill in &card.skills {
        parts.push(skill.name.clone());
        parts.push(skill.description.clone());
        if !skill.tags.is_empty() {
            parts.push(skill.tags.join(" "));
        }
        if !skill.examples.is_empty() {
            parts.push(skill.examples.join(" "));
        }
    }
    parts.join(". ")
}

#[derive(Debug)]
pub struct AgentSearchResult {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub agent_info: String,
}

fn extract_results(batches: Vec<RecordBatch>) -> Vec<AgentSearchResult> {
    let mut results = Vec::new();
    for batch in batches {
        let Some(id_col) = batch
            .column_by_name("agent_id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        else {
            continue;
        };

        let Some(name_col) = batch
            .column_by_name("name")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        else {
            continue;
        };

        let Some(desc_col) = batch
            .column_by_name("description")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        else {
            continue;
        };

        let Some(info_col) = batch
            .column_by_name("agent_info")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        else {
            continue;
        };

        for i in 0..batch.num_rows() {
            results.push(AgentSearchResult {
                agent_id: id_col.value(i).to_string(),
                name: name_col.value(i).to_string(),
                description: desc_col.value(i).to_string(),
                agent_info: info_col.value(i).to_string(),
            });
        }
    }
    results
}
