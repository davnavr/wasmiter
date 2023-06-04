use wasmiter::parser;

pub fn leb128(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("leb128");

    const BYTES: &[&[u8]] = &[
        &[0x10],
        &[0x80, 0x10],
        &[0x80, 0x80, 0x10],
        &[0x80, 0x80, 0x80, 0x10],
        &[0x80, 0x80, 0x80, 0x80, 0x02],
    ];

    for input in BYTES {
        group.throughput(criterion::Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(input.len()),
            input,
            |b, slice| {
                b.iter(|| parser::leb128::u32(&mut 0, *slice));
            },
        );
    }

    group.finish()
}

criterion::criterion_group!(benchmarks, leb128);
criterion::criterion_main!(benchmarks);
