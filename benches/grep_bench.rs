// This file is part of the uutils grep package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use uu_grep::matcher::Matcher;
use uu_grep::{BinaryMode, ColorConfig, Config, DeviceMode, DirectoryMode, GlobSet, RegexMode};

fn make_config<'a>(
    patterns: &'a [&'a str],
    regex_mode: RegexMode,
    ignore_case: bool,
    invert_match: bool,
    word_regexp: bool,
) -> Config<'a> {
    Config {
        directory_mode: DirectoryMode::Read,
        device_mode: DeviceMode::Default,
        follow_symlinks: false,
        include_globs: GlobSet::new(),
        exclude_globs: GlobSet::new(),
        exclude_dir_globs: GlobSet::new(),
        label: "(standard input)",
        #[cfg(windows)]
        strip_cr: false,
        binary_mode: BinaryMode::Binary,
        max_count: None,
        before_context: 0,
        after_context: 0,
        has_context: false,
        patterns,
        regex_mode,
        ignore_case,
        invert_match,
        word_regexp,
        line_regexp: false,
        quiet: true,
        count: false,
        show_filename: false,
        files_with_matches: false,
        files_without_match: false,
        only_matching: false,
        byte_offset: false,
        line_number: false,
        initial_tab: false,
        null_separator: false,
        null_data: false,
        line_buffered: false,
        no_messages: true,
        group_separator: None,
        use_color: false,
        color_config: ColorConfig {
            matched_selected: "",
            matched_context: "",
            filename: "",
            line_number: "",
            byte_offset: "",
            separator: "",
            selected_line: "",
            context_line: "",
            reverse_video: false,
            no_erase: false,
        },
    }
}

fn bench_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    group.bench_function("fixed_string", |b| {
        b.iter(|| {
            let patterns: &[&str] = &["hello world"];
            let config = make_config(patterns, RegexMode::Fixed, false, false, false);
            let matcher = Matcher::compile(black_box(&config)).unwrap();
            let _ = black_box(&matcher);
        })
    });

    group.bench_function("basic_regex", |b| {
        b.iter(|| {
            let patterns: &[&str] = &[r"[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}"];
            let config = make_config(patterns, RegexMode::Basic, false, false, false);
            let matcher = Matcher::compile(black_box(&config)).unwrap();
            let _ = black_box(&matcher);
        })
    });

    group.bench_function("extended_regex", |b| {
        b.iter(|| {
            let patterns: &[&str] = &[r"[0-9]{4}-[0-9]{2}-[0-9]{2}"];
            let config = make_config(patterns, RegexMode::Extended, false, false, false);
            let matcher = Matcher::compile(black_box(&config)).unwrap();
            let _ = black_box(&matcher);
        })
    });

    group.bench_function("perl_regex", |b| {
        b.iter(|| {
            let patterns: &[&str] = &[r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}"];
            let config = make_config(patterns, RegexMode::Perl, false, false, false);
            let matcher = Matcher::compile(black_box(&config)).unwrap();
            let _ = black_box(&matcher);
        })
    });

    group.bench_function("multiple_patterns", |b| {
        b.iter(|| {
            let patterns: &[&str] = &["error", "warning", "critical", "fatal", "panic"];
            let config = make_config(patterns, RegexMode::Fixed, false, false, false);
            let matcher = Matcher::compile(black_box(&config)).unwrap();
            let _ = black_box(&matcher);
        })
    });

    group.finish();
}

fn bench_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("match");

    // Fixed string match - hit
    {
        let patterns: &[&str] = &["ERROR"];
        let config = make_config(patterns, RegexMode::Fixed, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 ERROR: Connection timeout on server-42";

        group.bench_function("fixed_string_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Fixed string match - miss
    {
        let patterns: &[&str] = &["CRITICAL"];
        let config = make_config(patterns, RegexMode::Fixed, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 INFO: Server started successfully";

        group.bench_function("fixed_string_miss", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Extended regex match
    {
        let patterns: &[&str] = &[r"[0-9]{4}-[0-9]{2}-[0-9]{2}"];
        let config = make_config(patterns, RegexMode::Extended, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 ERROR: Connection timeout";

        group.bench_function("extended_regex_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Case-insensitive match
    {
        let patterns: &[&str] = &["error"];
        let config = make_config(patterns, RegexMode::Fixed, true, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 ERROR: Connection timeout";

        group.bench_function("case_insensitive_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Inverted match
    {
        let patterns: &[&str] = &["ERROR"];
        let config = make_config(patterns, RegexMode::Fixed, false, true, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 INFO: Server started successfully";

        group.bench_function("inverted_match", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Word boundary match
    {
        let patterns: &[&str] = &["error"];
        let config = make_config(patterns, RegexMode::Fixed, true, false, true);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 error: Connection timeout";

        group.bench_function("word_boundary_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Multiple patterns
    {
        let patterns: &[&str] = &["error", "warning", "critical", "fatal", "panic"];
        let config = make_config(patterns, RegexMode::Fixed, true, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let line = b"2024-01-15 10:30:45 WARNING: High memory usage detected on node-7";

        group.bench_function("multi_pattern_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(line))))
        });
    }

    // Long line
    {
        let patterns: &[&str] = &["needle"];
        let config = make_config(patterns, RegexMode::Fixed, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();
        let mut long_line = "a".repeat(5000);
        long_line.push_str("needle");
        long_line.push_str(&"b".repeat(5000));
        let long_line_bytes = long_line.into_bytes();

        group.bench_function("long_line_hit", |b| {
            b.iter(|| black_box(matcher.match_line(black_box(&long_line_bytes))))
        });
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Simulate processing many lines (like searching a log file)
    let lines: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            if i % 50 == 0 {
                format!(
                    "2024-01-15 10:30:{:02} ERROR: Connection timeout on server-{}",
                    i % 60,
                    i
                )
                .into_bytes()
            } else {
                format!(
                    "2024-01-15 10:30:{:02} INFO: Request processed in {}ms",
                    i % 60,
                    i * 3
                )
                .into_bytes()
            }
        })
        .collect();

    {
        let patterns: &[&str] = &["ERROR"];
        let config = make_config(patterns, RegexMode::Fixed, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();

        group.bench_function("scan_1000_lines_fixed", |b| {
            b.iter(|| {
                let mut matches = 0u64;
                for line in &lines {
                    if matcher.match_line(black_box(line)).is_some() {
                        matches += 1;
                    }
                }
                black_box(matches)
            })
        });
    }

    {
        let patterns: &[&str] = &[r"[0-9]+ *ms"];
        let config = make_config(patterns, RegexMode::Extended, false, false, false);
        let matcher = Matcher::compile(&config).unwrap();

        group.bench_function("scan_1000_lines_regex", |b| {
            b.iter(|| {
                let mut matches = 0u64;
                for line in &lines {
                    if matcher.match_line(black_box(line)).is_some() {
                        matches += 1;
                    }
                }
                black_box(matches)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_compile, bench_match, bench_throughput);
criterion_main!(benches);
