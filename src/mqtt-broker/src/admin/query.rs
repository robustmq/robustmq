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

use protocol::broker::broker_mqtt_admin::{MatchMode, OrderDirection, QueryOptions};

/// A common interface for resource types to support generic querying logic.
///
/// The `Queryable` trait provides a uniform way to retrieve a named field’s
/// string representation, enabling shared implementations of filtering,
/// sorting, and pagination across different data types (e.g., `SessionRaw`,
/// `ClientRaw`, etc.). By mapping field names to `Option<String>`, each
/// implementer can handle optional fields and custom formatting.
///
/// Combined with the `apply_filters`, `apply_sorting`, and `apply_pagination`
/// functions (driven by protobuf-defined `QueryOptions`, `MatchMode`, and
/// `OrderDirection`), this design centralizes all query-related code:
/// - **Filtering** handles exact and fuzzy matching (with room for future
///   extensions to support additional matching strategies).
/// - **Sorting** applies a dynamic `order_by` field and ascending/descending
///   direction.
/// - **Pagination** returns a sublist and total count based on `limit` and
///   `offset`.
///
/// This approach greatly reduces duplication in each endpoint handler and
/// makes it trivial to add new queryable types—simply implement `Queryable`.
pub trait Queryable {
    /// given the field name, return the field's string representation or None
    fn get_field_str(&self, field: &str) -> Option<String>;
}

struct FilterSpec {
    field: String,
    values: Vec<String>,
    mode: MatchMode,
}

pub fn apply_filters<T: Queryable>(items: Vec<T>, options: &Option<QueryOptions>) -> Vec<T> {
    let Some(qo) = options else { return items };
    if qo.filters.is_empty() {
        return items;
    }

    //  turn the filter spec into a vector of FilterSpec
    let mut specs = Vec::with_capacity(qo.filters.len());
    for f in &qo.filters {
        let values = f.values.iter().map(|v| v.to_lowercase()).collect();
        let mode = if let Some(raw_i) = f.exact_match {
            MatchMode::try_from(raw_i).unwrap_or(MatchMode::Exact)
        } else {
            MatchMode::Exact
        };
        specs.push(FilterSpec {
            field: f.field.clone(),
            values,
            mode,
        });
    }

    items
        .into_iter()
        .filter(|item| {
            // for every item, check all the filter spec
            for spec in &specs {
                match item.get_field_str(&spec.field) {
                    None if spec.values.is_empty() => continue,
                    None => return false,
                    Some(raw) => {
                        let raw = raw.to_lowercase();
                        // want to match None, but got Some, no match
                        if spec.values.is_empty() {
                            return false;
                        }
                        let ok = match spec.mode {
                            MatchMode::Exact => spec.values.iter().any(|v| &raw == v),
                            MatchMode::Fuzzy => spec.values.iter().any(|v| raw.contains(v)),
                        };
                        if !ok {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .collect()
}

pub fn apply_sorting<T: Queryable>(mut items: Vec<T>, options: &Option<QueryOptions>) -> Vec<T> {
    if let Some(opts) = options {
        if let Some(sorting) = &opts.sorting {
            let order_by = &sorting.order_by;
            let raw_dir = sorting.direction;
            let direction = OrderDirection::try_from(raw_dir).unwrap_or(OrderDirection::Asc);

            items.sort_by(|a, b| {
                let oa = a.get_field_str(order_by);
                let ob = b.get_field_str(order_by);

                let ord = oa.cmp(&ob);
                if direction == OrderDirection::Desc {
                    ord.reverse()
                } else {
                    ord
                }
            });
        }
    }
    items
}

pub fn apply_pagination<T: Queryable>(
    items: Vec<T>,
    options: &Option<QueryOptions>,
) -> (Vec<T>, usize) {
    let total_count = items.len();
    if let Some(opts) = options {
        if let Some(p) = &opts.pagination {
            let limit = p.limit;
            let offset = p.offset;
            let paginated = items
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect::<Vec<T>>();

            (paginated, total_count)
        } else {
            (items, total_count)
        }
    } else {
        (items, total_count)
    }
}
