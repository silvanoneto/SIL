//! Integration tests for LIS → JSIL compilation

#[cfg(feature = "jsil")]
mod jsil_tests {
    use lis_core::{compile_to_jsil, Lexer, Parser, Compiler};
    use sil_core::io::jsil::{JsilReader, CompressionMode};
    use sil_core::io::jsonl::JsonlRecord;
    use std::fs;

    #[test]
    fn test_compile_simple_to_jsil() {
        let source = r#"
            fn main() {
                let x = 42;
                return x;
            }
        "#;

        let output = "/tmp/test_simple.jsil";
        let stats = compile_to_jsil(source, output, None).unwrap();

        assert!(stats.record_count > 0, "Should have records");
        assert!(stats.compressed_size > 0, "Should have compressed data");

        // Verify file exists
        assert!(std::path::Path::new(output).exists());

        // Verify can be read
        let mut reader = JsilReader::load(output).unwrap();
        let mut meta_found = false;
        let mut symbol_found = false;
        let mut instr_found = false;

        while let Some(record) = reader.next_record::<JsonlRecord>().unwrap() {
            match record {
                JsonlRecord::Metadata { version, mode, .. } => {
                    meta_found = true;
                    assert_eq!(version, "1.0");
                    assert_eq!(mode, "Sil128");
                }
                JsonlRecord::Symbol { name, kind, .. } if name == "main" => {
                    symbol_found = true;
                    assert_eq!(kind, "function");
                }
                JsonlRecord::Instruction { .. } => {
                    instr_found = true;
                }
                _ => {}
            }
        }

        assert!(meta_found, "Should have metadata record");
        assert!(symbol_found, "Should have main symbol");
        assert!(instr_found, "Should have instructions");

        // Cleanup
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_compile_with_arithmetic() {
        let source = r#"
            fn calculate() {
                let a = 10;
                let b = 20;
                let c = a + b;
                return c;
            }
        "#;

        let output = "/tmp/test_arithmetic.jsil";
        let stats = compile_to_jsil(source, output, Some(CompressionMode::XorRotate)).unwrap();

        assert!(stats.record_count >= 3, "Should have multiple records");

        // Verify instructions include ADD
        let mut reader = JsilReader::load(output).unwrap();
        let mut add_found = false;

        while let Some(record) = reader.next_record::<JsonlRecord>().unwrap() {
            if let JsonlRecord::Instruction { op, .. } = record {
                if op == "ADD" {
                    add_found = true;
                }
            }
        }

        assert!(add_found, "Should have ADD instruction");

        // Cleanup
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_compression_modes() {
        let source = "fn main() { let x = 1 + 2; }";

        for mode in [
            CompressionMode::None,
            CompressionMode::Xor,
            CompressionMode::XorRotate,
        ] {
            let output = format!("/tmp/test_{:?}.jsil", mode);
            let stats = compile_to_jsil(source, &output, Some(mode)).unwrap();

            assert!(stats.record_count > 0, "Mode {:?} should produce records", mode);
            assert!(std::path::Path::new(&output).exists(), "Mode {:?} should create file", mode);

            // Verify can be read back
            let reader = JsilReader::load(&output).unwrap();
            assert_eq!(reader.header().compression, mode);

            // Cleanup
            let _ = fs::remove_file(&output);
        }
    }

    #[test]
    fn test_multiple_functions() {
        let source = r#"
            fn helper() {
                return 42;
            }

            fn main() {
                let x = helper();
                return x;
            }
        "#;

        let output = "/tmp/test_multi_func.jsil";
        let stats = compile_to_jsil(source, output, None).unwrap();

        // Verify we have both symbols
        let mut reader = JsilReader::load(output).unwrap();
        let mut helper_found = false;
        let mut main_found = false;

        while let Some(record) = reader.next_record::<JsonlRecord>().unwrap() {
            if let JsonlRecord::Symbol { name, .. } = record {
                if name == "helper" {
                    helper_found = true;
                }
                if name == "main" {
                    main_found = true;
                }
            }
        }

        assert!(helper_found, "Should have helper symbol");
        assert!(main_found, "Should have main symbol");

        // Cleanup
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_control_flow() {
        let source = r#"
            fn main() {
                let x = 0;
                if x == 0 {
                    x = 1;
                }
                return x;
            }
        "#;

        let output = "/tmp/test_control_flow.jsil";
        let stats = compile_to_jsil(source, output, None).unwrap();

        // Verify we have jump instructions
        let mut reader = JsilReader::load(output).unwrap();
        let mut jump_found = false;

        while let Some(record) = reader.next_record::<JsonlRecord>().unwrap() {
            if let JsonlRecord::Instruction { op, .. } = record {
                if op == "JZ" || op == "JMP" {
                    jump_found = true;
                }
            }
        }

        assert!(jump_found, "Should have jump instructions for if statement");

        // Cleanup
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_stats_report() {
        let source = "fn main() { let x = 42; }";
        let output = "/tmp/test_stats.jsil";
        let stats = compile_to_jsil(source, output, None).unwrap();

        let report = stats.report();
        assert!(report.contains("Records:"));
        assert!(report.contains("Uncompressed:"));
        assert!(report.contains("Compressed:"));
        assert!(report.contains("Ratio:"));

        // Cleanup
        let _ = fs::remove_file(output);
    }
}
