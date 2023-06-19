use arbitrary::{Arbitrary as _, Unstructured};
use criterion::{BatchSize, BenchmarkId};
use rand::RngCore as _;
use wasm_smith::Module;

fn printing(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("printing");

    fn iter_wat<'a, T>(
        unstructured_buffer: &'a mut [u8],
        mut f: impl FnMut(Vec<u8>) -> T + 'a,
    ) -> impl FnMut(&mut criterion::Bencher, &usize) + 'a {
        move |b, size| {
            let unstructured = &mut unstructured_buffer[..*size];
            let mut rng = rand::thread_rng();
            let mut make_module = || {
                rng.fill_bytes(unstructured);
                Module::arbitrary_take_rest(Unstructured::new(unstructured))
                    .unwrap()
                    .to_bytes()
            };

            b.iter_batched(&mut make_module, &mut f, BatchSize::SmallInput)
        }
    }

    let unstructured_sizes = [0x40000usize, 0x40000000];
    let mut unstructured_buffer = vec![0u8; *unstructured_sizes.last().unwrap()];
    for size in unstructured_sizes {
        group.throughput(criterion::Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("wasmiter", size),
            &size,
            iter_wat(&mut unstructured_buffer, |wasm| {
                format!(
                    "{}",
                    wasmiter::parse_module_sections(wasm.as_slice())
                        .unwrap()
                        .display_module()
                );
            }),
        );
        group.bench_with_input(
            BenchmarkId::new("wasmprinter", size),
            &size,
            iter_wat(&mut unstructured_buffer, wasmprinter::print_bytes),
        );
    }

    group.finish();
}

criterion::criterion_group!(benchmarks, printing);
criterion::criterion_main!(benchmarks);
