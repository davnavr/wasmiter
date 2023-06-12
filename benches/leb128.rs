//! Benchmarking of LEB128 parsers.

pub fn s32(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("s32");

    const BITS: core::ops::RangeInclusive<i32> = 0..=31;
    const PERMUTATION_LEN: usize = 5;
    const COUNT: usize = (*BITS.end() as usize + 1) * PERMUTATION_LEN;

    let mut input = Vec::<u8>::with_capacity(COUNT);

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

    group.throughput(criterion::Throughput::Bytes(input.len() as u64));
    group.bench_with_input("wasmiter", input.as_slice(), |b, input| {
        b.iter(|| {
            let mut offset = 0u64;
            for _ in 0..COUNT {
                criterion::black_box(wasmiter::parser::leb128::s32(&mut offset, input).unwrap());
            }
        })
    });
    group.bench_with_input("leb128", input.as_slice(), |b, input| {
        b.iter(|| {
            let mut bytes = input;
            for _ in 0..COUNT {
                criterion::black_box(leb128::read::signed(&mut bytes).unwrap());
            }
        })
    });

    group.finish()
}

criterion::criterion_group!(benchmarks, s32);
criterion::criterion_main!(benchmarks);
