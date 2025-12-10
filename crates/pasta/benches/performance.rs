//! Performance benchmarks for Pasta script engine.
//!
//! These benchmarks verify that the engine meets the performance requirements:
//! - Parse 1000-line script in <100ms
//! - Execute labels 100 times continuously with acceptable performance

use pasta::PastaEngine;
use std::time::Instant;

/// Generate a large script with many labels for testing.
///
/// Creates a script with the specified number of labels, each containing
/// multiple lines of dialogue.
fn generate_large_script(num_labels: usize, lines_per_label: usize) -> String {
    let mut script = String::new();

    for i in 0..num_labels {
        script.push_str(&format!("＊ラベル{}\n", i));
        script.push_str("    ＠time：morning\n");

        for j in 0..lines_per_label {
            let speaker = if j % 2 == 0 {
                "さくら"
            } else {
                "うにゅう"
            };
            script.push_str(&format!(
                "    {}：これは{}行目のテキストです。\n",
                speaker,
                j + 1
            ));
        }

        script.push('\n');
    }

    script
}

/// Benchmark: Parse a 1000-line script.
///
/// Requirement: Should complete in less than 100ms.
fn benchmark_large_script_parse() {
    println!("\n=== Benchmark: Large Script Parsing ===");

    // Generate a script with approximately 1000 lines
    // 100 labels * 10 lines each = 1000 lines
    let script = generate_large_script(100, 10);
    let line_count = script.lines().count();

    println!("Script size: {} lines, {} bytes", line_count, script.len());

    // First parse (cold - no cache)
    PastaEngine::clear_cache();
    let start = Instant::now();
    let engine_result = PastaEngine::new(&script);
    let duration = start.elapsed();

    assert!(engine_result.is_ok(), "Failed to parse script");
    println!("First parse (cold):  {:?}", duration);

    if duration.as_millis() > 100 {
        println!("⚠️  WARNING: Parse time exceeds 100ms requirement");
    } else {
        println!("✓ Parse time within 100ms requirement");
    }

    // Second parse (warm - with cache)
    let start = Instant::now();
    let engine_result2 = PastaEngine::new(&script);
    let duration2 = start.elapsed();

    assert!(engine_result2.is_ok(), "Failed to parse script (cached)");
    println!("Second parse (warm): {:?}", duration2);

    // Cache should be significantly faster
    let speedup = duration.as_micros() as f64 / duration2.as_micros() as f64;
    println!("Cache speedup: {:.2}x", speedup);

    if speedup > 10.0 {
        println!("✓ Cache provides significant speedup (>10x)");
    } else {
        println!("⚠️  Cache speedup less than expected");
    }

    println!("Cache size: {} entries", PastaEngine::cache_size());
}

/// Benchmark: Execute labels 100 times.
///
/// Tests label lookup performance and execution overhead.
fn benchmark_label_execution() {
    println!("\n=== Benchmark: Label Execution (100 iterations) ===");

    let script = r#"
＊挨拶
    さくら：こんにちは！
    うにゅう：やあ、元気？
    さくら：うん、元気だよ！

＊別れ
    さくら：それじゃあ、またね！
    うにゅう：バイバイ！

＊質問
    さくら：何か質問ある？
    うにゅう：特にないよ。
    さくら：そっか。
"#;

    let mut engine = PastaEngine::new(script).expect("Failed to create engine");

    // Warm-up
    for _ in 0..10 {
        let _ = engine.execute_label("挨拶");
    }

    // Benchmark 100 executions
    let start = Instant::now();
    for i in 0..100 {
        let label = match i % 3 {
            0 => "挨拶",
            1 => "別れ",
            _ => "質問",
        };

        let events = engine
            .execute_label(label)
            .expect("Failed to execute label");
        assert!(!events.is_empty(), "No events generated");
    }
    let duration = start.elapsed();

    println!("100 label executions: {:?}", duration);
    println!("Average per execution: {:?}", duration / 100);

    let avg_micros = duration.as_micros() / 100;
    if avg_micros < 1000 {
        println!("✓ Average execution time < 1ms");
    } else {
        println!("⚠️  Average execution time: {}μs", avg_micros);
    }
}

/// Benchmark: Label lookup performance.
///
/// Tests the HashMap-based label lookup with many labels.
fn benchmark_label_lookup() {
    println!("\n=== Benchmark: Label Lookup Performance ===");

    // Create script with 1000 labels
    let script = generate_large_script(1000, 2);
    let engine = PastaEngine::new(&script).expect("Failed to create engine");

    println!("Total labels: {}", engine.label_names().len());

    // Benchmark label existence checks
    let start = Instant::now();
    for i in 0..1000 {
        let label_name = format!("ラベル{}", i);
        assert!(
            engine.has_label(&label_name),
            "Label not found: {}",
            label_name
        );
    }
    let duration = start.elapsed();

    println!("1000 label lookups: {:?}", duration);
    println!("Average per lookup: {:?}", duration / 1000);

    let avg_nanos = duration.as_nanos() / 1000;
    if avg_nanos < 1000 {
        println!("✓ Average lookup time < 1μs (O(1) performance)");
    } else {
        println!("⚠️  Average lookup time: {}ns", avg_nanos);
    }
}

/// Benchmark: Multiple duplicate labels with random selection.
///
/// Tests performance when multiple labels share the same name.
fn benchmark_duplicate_labels() {
    println!("\n=== Benchmark: Duplicate Label Selection ===");

    let mut script = String::new();

    // Create 100 labels with the same name
    for i in 0..100 {
        script.push_str("＊挨拶\n");
        script.push_str(&format!("    さくら：挨拶バリエーション{}\n", i));
        script.push('\n');
    }

    let mut engine = PastaEngine::new(&script).expect("Failed to create engine");

    println!("Labels with name '挨拶': 100");

    // Benchmark 100 executions with random selection
    let start = Instant::now();
    for _ in 0..100 {
        let events = engine
            .execute_label("挨拶")
            .expect("Failed to execute label");
        assert!(!events.is_empty(), "No events generated");
    }
    let duration = start.elapsed();

    println!("100 executions with random selection: {:?}", duration);
    println!("Average per execution: {:?}", duration / 100);

    let avg_micros = duration.as_micros() / 100;
    if avg_micros < 5000 {
        println!("✓ Random selection performance acceptable");
    } else {
        println!("⚠️  Random selection overhead: {}μs", avg_micros);
    }
}

/// Benchmark: Event generation throughput.
///
/// Tests how many events can be generated per second.
fn benchmark_event_throughput() {
    println!("\n=== Benchmark: Event Generation Throughput ===");

    let script = r#"
＊test
    さくら：１行目
    さくら：２行目
    さくら：３行目
    さくら：４行目
    さくら：５行目
    さくら：６行目
    さくら：７行目
    さくら：８行目
    さくら：９行目
    さくら：１０行目
"#;

    let mut engine = PastaEngine::new(script).expect("Failed to create engine");

    let mut total_events = 0;
    let start = Instant::now();

    // Generate events for 1 second
    let duration_limit = std::time::Duration::from_secs(1);
    let mut iterations = 0;

    while start.elapsed() < duration_limit {
        let events = engine
            .execute_label("test")
            .expect("Failed to execute label");
        total_events += events.len();
        iterations += 1;
    }

    let duration = start.elapsed();

    println!("Generated {} events in {:?}", total_events, duration);
    println!("Iterations: {}", iterations);
    println!(
        "Events per second: {:.0}",
        total_events as f64 / duration.as_secs_f64()
    );

    let events_per_sec = total_events as f64 / duration.as_secs_f64();
    if events_per_sec > 10000.0 {
        println!("✓ High throughput (>10k events/sec)");
    } else {
        println!("⚠️  Throughput: {:.0} events/sec", events_per_sec);
    }
}

/// Benchmark: Cache efficiency with varied scripts.
///
/// Tests cache hit rate and memory usage with different scripts.
fn benchmark_cache_efficiency() {
    println!("\n=== Benchmark: Cache Efficiency ===");

    // Clear cache
    PastaEngine::clear_cache();

    // Create 10 different scripts
    let scripts: Vec<String> = (0..10)
        .map(|i| generate_large_script(50 + i * 10, 5))
        .collect();

    // First pass - all cache misses
    let start = Instant::now();
    for script in &scripts {
        let _ = PastaEngine::new(script).expect("Failed to create engine");
    }
    let duration_cold = start.elapsed();

    println!("First pass (cold): {:?}", duration_cold);
    println!("Cache size: {}", PastaEngine::cache_size());

    // Second pass - all cache hits
    let start = Instant::now();
    for script in &scripts {
        let _ = PastaEngine::new(script).expect("Failed to create engine");
    }
    let duration_warm = start.elapsed();

    println!("Second pass (warm): {:?}", duration_warm);

    let speedup = duration_cold.as_micros() as f64 / duration_warm.as_micros() as f64;
    println!("Overall speedup: {:.2}x", speedup);

    if speedup > 5.0 {
        println!("✓ Cache provides excellent speedup (>5x)");
    } else {
        println!("⚠️  Cache speedup less than expected");
    }
}

fn main() {
    println!("Pasta Script Engine Performance Benchmarks");
    println!("===========================================");

    benchmark_large_script_parse();
    benchmark_label_execution();
    benchmark_label_lookup();
    benchmark_duplicate_labels();
    benchmark_event_throughput();
    benchmark_cache_efficiency();

    println!("\n===========================================");
    println!("All benchmarks completed!");
}
