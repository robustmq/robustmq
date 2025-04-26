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

use protocol::broker_mqtt::broker_mqtt_admin::{
    ClientRaw, MatchMode, OrderDirection, QueryOptions, SessionRaw,
};

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
/// - Filtering handles exact and fuzzy matching (with room for future
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

impl Queryable for SessionRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "session_expiry" => Some(self.session_expiry.to_string()),
            "is_contain_last_will" => Some(self.is_contain_last_will.to_string()),
            "last_will_delay_interval" => self.last_will_delay_interval.map(|v| v.to_string()),
            "create_time" => Some(self.create_time.to_string()),
            "connection_id" => self.connection_id.map(|v| v.to_string()),
            "broker_id" => self.broker_id.map(|v| v.to_string()),
            "reconnect_time" => self.reconnect_time.map(|v| v.to_string()),
            "distinct_time" => self.distinct_time.map(|v| v.to_string()),
            _ => None,
        }
    }
}

impl Queryable for ClientRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "username" => Some(self.username.clone()),
            "is_online" => Some(self.is_online.to_string()),
            "source_ip" => Some(self.source_ip.clone()),
            "connected_at" => Some(self.connected_at.to_string()),
            "keep_alive" => Some(self.keep_alive.to_string()),
            "clean_session" => Some(self.clean_session.to_string()),
            "session_expiry_interval" => Some(self.session_expiry_interval.to_string()),
            _ => None,
        }
    }
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
            MatchMode::try_from(raw_i).unwrap_or_else(|_| MatchMode::Exact)
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
            if let Some(order_by) = &sorting.order_by {
                let direction = if let Some(raw_dir) = sorting.direction {
                    OrderDirection::try_from(raw_dir).unwrap_or(OrderDirection::Asc)
                } else {
                    OrderDirection::Asc
                };

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
    }
    items
}

pub fn apply_pagination<T>(items: Vec<T>, options: &Option<QueryOptions>) -> (Vec<T>, usize) {
    // default pagination limit: 10
    let default_limit: u32 = 10;
    let default_offset: u32 = 0;

    let total_count = items.len();

    let (limit, offset) = if let Some(qo) = options {
        if let Some(p) = &qo.pagination {
            (
                p.limit.unwrap_or(default_limit),
                p.offset.unwrap_or(default_offset),
            )
        } else {
            (default_limit, default_offset)
        }
    } else {
        (default_limit, default_offset)
    };

    let paginated = items
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect::<Vec<T>>();

    (paginated, total_count)
}
