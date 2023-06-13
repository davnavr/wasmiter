//! Benchmarking of LEB128 parsers.

use criterion::{black_box, Bencher, BenchmarkId};

fn mixed_size_input() -> Vec<u8> {
    const BITS: core::ops::RangeInclusive<i32> = 0..=31;
    const PERMUTATION_LEN: usize = 5;

    let mut input = Vec::<u8>::with_capacity((*BITS.end() as usize + 1) * 2 * PERMUTATION_LEN);

    BITS.map(|i| 1i32 << i)
        .flat_map::<[i32; PERMUTATION_LEN], _>(|i| {
            [
                i.wrapping_sub(2),
                i.wrapping_sub(1),
                i,
                i.wrapping_add(1),
                i.wrapping_add(2),
            ]
        })
        .for_each(|i| {
            leb128::write::signed(&mut input, i.into()).unwrap();
        });

    input
}

fn small_input() -> Vec<u8> {
    const RANGE: core::ops::RangeInclusive<i8> = i8::MIN..=i8::MAX;
    const PERMUTATION_LEN: usize = 3;

    let mut input = Vec::<u8>::with_capacity(*RANGE.end() as usize * 2 * PERMUTATION_LEN);

    RANGE
        .flat_map::<[i32; PERMUTATION_LEN], _>(|b| {
            let i = i32::from(b);
            [i.wrapping_sub(1), i, i.wrapping_add(i)]
        })
        .for_each(|i| {
            leb128::write::signed(&mut input, i.into()).unwrap();
        });

    input
}

pub fn s32(c: &mut criterion::Criterion) {
    fn iter_wasmiter_simple(b: &mut Bencher<'_>, input: &[u8]) {
        b.iter(|| {
            let mut offset = 0u64;
            while offset < input.len() as u64 {
                black_box(wasmiter::parser::leb128::simple::s32(&mut offset, input).unwrap());
            }
        })
    }

    fn iter_leb128(b: &mut Bencher<'_>, input: &[u8]) {
        b.iter(|| {
            let mut bytes = input;
            while !bytes.is_empty() {
                black_box(leb128::read::signed::<&[u8]>(&mut bytes).unwrap());
            }
        })
    }

    fn group_with_input(
        input_name: &str,
        input: &[u8],
        group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    ) {
        group.throughput(criterion::Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("wasmiter", input_name),
            input,
            |b, input| iter_wasmiter_simple(b, input),
        );
        group.bench_with_input(BenchmarkId::new("leb128", input_name), input, |b, input| {
            iter_leb128(b, input)
        });
    }

    let mut group = c.benchmark_group("s32");

    let mixed_size_bytes = mixed_size_input();
    group_with_input(stringify!(mixed_size_bytes), &mixed_size_bytes, &mut group);
    std::mem::drop(mixed_size_bytes);

    let small_bytes = small_input();
    group_with_input(stringify!(small_bytes), &small_bytes, &mut group);
    std::mem::drop(small_bytes);

    group.finish()
}

criterion::criterion_group!(benchmarks, s32);
criterion::criterion_main!(benchmarks);
