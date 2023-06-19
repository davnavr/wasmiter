//! Benchmarking of LEB128 parsers.

use criterion::{black_box, Bencher, BenchmarkId};

macro_rules! benchmark_parsers {
    ($(
        $name:ident => {
            $(
                let $input_set:ident = $input_set_body:expr;
            )+
            $(
                fn $iter_name:ident = |$iter_input_name:ident| $iter_body:expr;
            )+
        };
    )*) => {$(
        pub fn $name(c: &mut criterion::Criterion) {
            $(
                fn $iter_name(b: &mut Bencher<'_>, $iter_input_name: &[u8]) {
                    b.iter(|| $iter_body)
                }
            )+

            fn bench_set(
                input_name: &'static str,
                input: &[u8],
                group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
            ) {
                group.throughput(criterion::Throughput::Bytes(input.len() as u64));
                $(
                    group.bench_with_input(
                        BenchmarkId::new(stringify!($iter_name), input_name),
                        input,
                        |b, input| $iter_name(b, input),
                    );
                )+
            }

            let mut group = c.benchmark_group(stringify!($name));

            $({
                let $input_set: Vec<u8> = $input_set_body;
                bench_set(stringify!($input_set), $input_set.as_slice(), &mut group);
            })+

            group.finish()
        }
    )*};
}

benchmark_parsers! {
    s32 => {
        let mixed_size_input = {
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
        };

        let small_input = {
            const RANGE: core::ops::RangeInclusive<i8> = i8::MIN..=i8::MAX;
            const PERMUTATION_LEN: usize = 3;

            let mut input = Vec::<u8>::with_capacity(*RANGE.end() as usize * 2 * PERMUTATION_LEN);

            RANGE
                .flat_map::<[i32; PERMUTATION_LEN], _>(|b| {
                    [i32::from(b.wrapping_sub(1)), i32::from(b), i32::from(b.wrapping_add(1))]
                })
                .for_each(|i| {
                    leb128::write::signed(&mut input, i.into()).unwrap();
                });

            input
        };

        fn wasmiter_simple = |input| {
            let mut offset = 0u64;
            while offset < input.len() as u64 {
                black_box(wasmiter::parser::leb128::simple::s32(&mut offset, input).unwrap());
            }
        };

        fn leb128 = |input| {
            let mut bytes = input;
            while !bytes.is_empty() {
                black_box(leb128::read::signed::<&[u8]>(&mut bytes).unwrap());
            }
        };
    };

    u32 => {
        let mixed_size_input = {
            const BITS: core::ops::RangeInclusive<u8> = 0..=31;
            const PERMUTATION_LEN: usize = 5;

            let mut input = Vec::<u8>::with_capacity((*BITS.end() as usize + 1) * 2 * PERMUTATION_LEN);

            BITS.map(|i| 1u32 << i)
                .flat_map::<[u32; PERMUTATION_LEN], _>(|i| {
                    [
                        i.wrapping_sub(2),
                        i.wrapping_sub(1),
                        i,
                        i.wrapping_add(1),
                        i.wrapping_add(2),
                    ]
                })
                .for_each(|i| {
                    leb128::write::unsigned(&mut input, i.into()).unwrap();
                });

            input
        };

        let small_input = {
            const RANGE: core::ops::RangeInclusive<u32> = 1u32..=254;
            const PERMUTATION_LEN: usize = 4;

            let mut input = Vec::<u8>::with_capacity(*RANGE.end() as usize * 2 * PERMUTATION_LEN);

            RANGE
                .flat_map::<[u32; PERMUTATION_LEN], _>(|i| {
                    [i.wrapping_sub(1), i, i.wrapping_add(1), i.wrapping_add(2)]
                })
                .for_each(|i| {
                    leb128::write::unsigned(&mut input, i.into()).unwrap();
                });

            input
        };

        fn wasmiter_simple = |input| {
            let mut offset = 0u64;
            while offset < input.len() as u64 {
                black_box(wasmiter::parser::leb128::simple::u32(&mut offset, input).unwrap());
            }
        };

        fn leb128 = |input| {
            let mut bytes = input;
            while !bytes.is_empty() {
                black_box(leb128::read::unsigned::<&[u8]>(&mut bytes).unwrap());
            }
        };
    };
}

criterion::criterion_group!(benchmarks, s32, u32);
criterion::criterion_main!(benchmarks);
