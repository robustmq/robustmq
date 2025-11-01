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

// Benchmark comparing JSON vs Bincode serialization
// Run with: cargo bench --bench serialization_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use metadata_struct::adapter::record::{Header, Record};

fn create_test_record(data_size: usize) -> Record {
    Record {
        offset: Some(12345),
        header: vec![
            Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            },
            Header {
                name: "device-id".to_string(),
                value: "sensor-001".to_string(),
            },
        ],
        key: "test-key-12345".to_string(),
        data: vec![0u8; data_size],
        tags: vec!["temperature".to_string(), "sensor".to_string(), "iot".to_string()],
        timestamp: 1699000000000,
        crc_num: 123456789,
    }
}

fn bench_json_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialize");
    
    for size in [128, 1024, 4096, 16384].iter() {
        let record = create_test_record(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _result = serde_json::to_string(black_box(&record)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_bincode_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("bincode_serialize");
    
    for size in [128, 1024, 4096, 16384].iter() {
        let record = create_test_record(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _result = bincode::serialize(black_box(&record)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_json_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_deserialize");
    
    for size in [128, 1024, 4096, 16384].iter() {
        let record = create_test_record(*size);
        let serialized = serde_json::to_string(&record).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _result: Record = serde_json::from_str(black_box(&serialized)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_bincode_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("bincode_deserialize");
    
    for size in [128, 1024, 4096, 16384].iter() {
        let record = create_test_record(*size);
        let serialized = bincode::serialize(&record).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _result: Record = bincode::deserialize(black_box(&serialized)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("size_comparison");
    
    for size in [128, 1024, 4096, 16384].iter() {
        let record = create_test_record(*size);
        
        let json_size = serde_json::to_string(&record).unwrap().len();
        let bincode_size = bincode::serialize(&record).unwrap().len();
        let savings = (1.0 - (bincode_size as f64 / json_size as f64)) * 100.0;
        
        println!("\nData size {}: JSON={} bytes, Bincode={} bytes, Savings={:.1}%", 
                 size, json_size, bincode_size, savings);
    }
    group.finish();
}

criterion_group!(
    benches, 
    bench_json_serialize,
    bench_bincode_serialize,
    bench_json_deserialize,
    bench_bincode_deserialize,
    bench_size_comparison
);
criterion_main!(benches);

