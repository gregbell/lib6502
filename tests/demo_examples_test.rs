//! Integration test to ensure all demo/examples/*.asm files can assemble successfully

use lib6502::assembler::assemble;
use std::fs;
use std::path::Path;

#[test]
fn test_all_demo_examples_assemble() {
    let demo_dir = Path::new("demo/examples");

    // Check if directory exists
    if !demo_dir.exists() {
        // If demo directory doesn't exist, skip this test
        println!("Warning: demo/examples directory not found, skipping test");
        return;
    }

    let mut files = Vec::new();

    // Collect all .asm files
    for entry in fs::read_dir(demo_dir).expect("Failed to read demo/examples directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("asm") {
            files.push(path);
        }
    }

    // Sort files for consistent test output
    files.sort();

    if files.is_empty() {
        println!("Warning: No .asm files found in demo/examples");
        return;
    }

    let mut failed_files = Vec::new();
    let mut succeeded_files = Vec::new();

    // Try to assemble each file
    for file_path in &files {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let source = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_name, e));

        match assemble(&source) {
            Ok(output) => {
                println!("✓ {} - assembled {} bytes", file_name, output.bytes.len());
                succeeded_files.push(file_name.to_string());
            }
            Err(errors) => {
                println!("✗ {} - failed with {} errors:", file_name, errors.len());
                for error in errors.iter().take(3) {
                    println!("  Line {}: {}", error.line, error.message);
                }
                if errors.len() > 3 {
                    println!("  ... and {} more errors", errors.len() - 3);
                }
                failed_files.push(file_name.to_string());
            }
        }
    }

    // Print summary
    println!("\n========================================");
    println!("Assembly Test Summary");
    println!("========================================");
    println!("Total files: {}", files.len());
    println!("Succeeded: {}", succeeded_files.len());
    println!("Failed: {}", failed_files.len());

    if !failed_files.is_empty() {
        println!("\nFailed files:");
        for file in &failed_files {
            println!("  - {}", file);
        }
    }

    // The test fails if any files failed to assemble
    assert!(
        failed_files.is_empty(),
        "Failed to assemble {} file(s): {}",
        failed_files.len(),
        failed_files.join(", ")
    );
}

#[test]
fn test_uart_hello_specifically() {
    let source =
        fs::read_to_string("demo/examples/uart-hello.asm").expect("Failed to read uart-hello.asm");

    let result = assemble(&source);

    match &result {
        Ok(output) => {
            println!(
                "uart-hello.asm assembled successfully: {} bytes",
                output.bytes.len()
            );

            // Verify the "Hello, 6502!" string is in the output
            let hello_str = b"Hello, 6502!";
            let found = output
                .bytes
                .windows(hello_str.len())
                .any(|window| window == hello_str);
            assert!(
                found,
                "Expected to find 'Hello, 6502!' string in assembled output"
            );
        }
        Err(errors) => {
            println!("uart-hello.asm failed with errors:");
            for error in errors {
                println!("  Line {}: {}", error.line, error.message);
            }
            panic!("uart-hello.asm should assemble without errors");
        }
    }
}

#[test]
fn test_uart_echo_specifically() {
    let source =
        fs::read_to_string("demo/examples/uart-echo.asm").expect("Failed to read uart-echo.asm");

    let result = assemble(&source);

    match &result {
        Ok(output) => {
            println!(
                "uart-echo.asm assembled successfully: {} bytes",
                output.bytes.len()
            );
            assert!(!output.bytes.is_empty(), "Expected non-empty output");
        }
        Err(errors) => {
            println!("uart-echo.asm failed with errors:");
            for error in errors {
                println!("  Line {}: {}", error.line, error.message);
            }
            panic!("uart-echo.asm should assemble without errors");
        }
    }
}

#[test]
fn test_uart_interrupt_advanced_specifically() {
    let source = fs::read_to_string("demo/examples/uart-interrupt-advanced.asm")
        .expect("Failed to read uart-interrupt-advanced.asm");

    let result = assemble(&source);

    match &result {
        Ok(output) => {
            println!(
                "uart-interrupt-advanced.asm assembled successfully: {} bytes",
                output.bytes.len()
            );
            assert!(!output.bytes.is_empty(), "Expected non-empty output");
        }
        Err(errors) => {
            println!("uart-interrupt-advanced.asm failed with errors:");
            for error in errors {
                println!("  Line {}: {}", error.line, error.message);
            }
            panic!("uart-interrupt-advanced.asm should assemble without errors");
        }
    }
}
