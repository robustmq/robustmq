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

#[derive(Clone, Debug)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub field: String,
    pub values: Vec<String>,
    pub exact_match: Option<MatchMode>,
}

#[derive(Clone, Debug)]
pub enum MatchMode {
    EXACT,
    FUZZY,
}

#[derive(Clone, Debug)]
pub struct Sorting {
    pub order_by: String,
    pub direction: OrderDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub enum OrderDirection {
    ASC,
    DESC,
}

#[derive(Clone, Debug)]
pub struct QueryOptions {
    pub pagination: Option<Pagination>,
    pub filters: Vec<Filter>,
    pub sorting: Option<Sorting>,
}

pub fn build_query_params(
    page: Option<u32>,
    limit: Option<u32>,
    sort_field: Option<String>,
    sort_by: Option<String>,
    filter_field: Option<String>,
    filter_values: Option<Vec<String>>,
    exact_match: Option<String>,
) -> Option<QueryOptions> {
    let pagination = Pagination {
        limit: parse_limit(limit),
        offset: parse_page(page),
    };
    let filters = parse_filters(filter_field, filter_values, exact_match);
    let sorting = parse_sorting(sort_field, sort_by);
    Some(QueryOptions {
        pagination: Some(pagination),
        filters,
        sorting,
    })
}

fn parse_filters(
    filter_field: Option<String>,
    filter_values: Option<Vec<String>>,
    exact_match: Option<String>,
) -> Vec<Filter> {
    let mut filters = Vec::new();
    if filter_field.is_none() || filter_values.is_none() {
        return filters;
    }

    let field = filter_field.unwrap();
    let values = filter_values.unwrap();
    if values.is_empty() {
        return filters;
    }

    filters.push(Filter {
        field,
        values,
        exact_match: Some(parse_match_mode(exact_match)),
    });
    filters
}

fn parse_match_mode(exact_match: Option<String>) -> MatchMode {
    if let Some(exact) = exact_match {
        if exact == *"exact" {
            return MatchMode::EXACT;
        }
        return MatchMode::FUZZY;
    }
    MatchMode::EXACT
}

fn parse_limit(page_num: Option<u32>) -> u32 {
    if let Some(num) = page_num {
        if num == 0 {
            return 10;
        } else {
            return num;
        }
    }
    10
}

fn parse_page(page: Option<u32>) -> u32 {
    if let Some(pg) = page {
        if pg == 0 {
            return 1;
        } else {
            return pg;
        }
    }
    1
}

fn parse_sorting(sort_field: Option<String>, sort_by: Option<String>) -> Option<Sorting> {
    if let Some(field) = sort_field {
        return Some(Sorting {
            order_by: field,
            direction: parse_order_by(sort_by),
        });
    }

    None
}

fn parse_order_by(sort_by: Option<String>) -> OrderDirection {
    if let Some(by) = sort_by {
        if by == *"asc" {
            return OrderDirection::ASC;
        }
    }
    OrderDirection::DESC
}

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
        let mode = if let Some(raw_i) = f.exact_match.clone() {
            raw_i
        } else {
            MatchMode::EXACT
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
                            MatchMode::EXACT => spec.values.iter().any(|v| &raw == v),
                            MatchMode::FUZZY => spec.values.iter().any(|v| raw.contains(v)),
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
            let raw_dir = sorting.direction.clone();

            items.sort_by(|a, b| {
                let oa = a.get_field_str(order_by);
                let ob = b.get_field_str(order_by);

                let ord = oa.cmp(&ob);
                if raw_dir == OrderDirection::DESC {
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
