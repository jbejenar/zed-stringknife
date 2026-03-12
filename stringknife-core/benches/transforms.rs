//! Performance benchmarks for StringKnife transforms.
//!
//! Validates the performance contract: < 100ms for 100KB input.
//! Input sizes: 1KB, 10KB, 100KB, 1MB (1MB tests InputTooLarge fast-path).

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use stringknife_core::transforms::{base64, case, hash, json};

/// Generate a repeating prose-like string of approximately `size` bytes.
fn generate_text(size: usize) -> String {
    let pattern = "The quick brown fox jumps over the lazy dog. ";
    let repeated = pattern.repeat(size / pattern.len() + 1);
    repeated[..size].to_string()
}

/// Generate a valid JSON object of approximately `size` bytes.
fn generate_json(size: usize) -> String {
    let mut s = String::with_capacity(size + 64);
    s.push('{');
    let mut i = 0;
    while s.len() < size.saturating_sub(10) {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "\"key_{i}\":\"value_{i} with some padding text to fill space\""
        ));
        i += 1;
    }
    s.push('}');
    s
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{}MB", bytes / 1_048_576)
    } else if bytes >= 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{bytes}B")
    }
}

const SIZES: [usize; 4] = [1_024, 10_240, 102_400, 1_048_576];

fn bench_base64_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64_encode");
    for &size in &SIZES {
        let input = generate_text(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(format_size(size)),
            &input,
            |b, input| {
                b.iter(|| base64::base64_encode(input));
            },
        );
    }
    group.finish();
}

fn bench_base64_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64_decode");
    for &size in &SIZES {
        let input = generate_text(size);
        // Encode first to get valid base64 to decode.
        // For 1MB, encode will return InputTooLarge — skip that size for decode.
        if let Ok(encoded) = base64::base64_encode(&input) {
            group.bench_with_input(
                BenchmarkId::from_parameter(format_size(size)),
                &encoded,
                |b, encoded| {
                    b.iter(|| base64::base64_decode(encoded));
                },
            );
        }
    }
    group.finish();
}

fn bench_sha256(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256");
    for &size in &SIZES {
        let input = generate_text(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(format_size(size)),
            &input,
            |b, input| {
                b.iter(|| hash::sha256(input));
            },
        );
    }
    group.finish();
}

fn bench_to_snake_case(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_snake_case");
    for &size in &SIZES {
        let input = generate_text(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(format_size(size)),
            &input,
            |b, input| {
                b.iter(|| case::to_snake_case(input));
            },
        );
    }
    group.finish();
}

fn bench_json_pretty_print(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_pretty_print");
    for &size in &SIZES {
        let input = generate_json(size);
        // 1MB JSON will hit InputTooLarge — benchmark the fast-path rejection too.
        group.bench_with_input(
            BenchmarkId::from_parameter(format_size(size)),
            &input,
            |b, input| {
                b.iter(|| json::json_pretty_print(input));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_base64_encode,
    bench_base64_decode,
    bench_sha256,
    bench_to_snake_case,
    bench_json_pretty_print,
);
criterion_main!(benches);
