use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn get_bin_path() -> PathBuf {
    // In a real scenario we might use assert_cmd or find the target dir.
    // simpler: usage cargo run.
    PathBuf::from("cargo")
}

#[test]
fn test_cli_generate_command() {
    let tmp_dir = env::temp_dir().join("oxidx_tests");
    fs::create_dir_all(&tmp_dir).unwrap();

    let input_path = tmp_dir.join("test_input.json");
    let output_path = tmp_dir.join("test_output.rs");

    let json_content = r#"{
        "type": "VStack",
        "children": [
            {
                "type": "Label",
                "props": { "text": "Integration Test" }
            }
        ]
    }"#;

    fs::write(&input_path, json_content).expect("Failed to write input file");

    let status = Command::new("cargo")
        .args(&[
            "run",
            "-q",
            "-p",
            "oxidx_cli",
            "--",
            "generate",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute oxidx cli");

    assert!(status.success(), "CLI command failed");
    assert!(output_path.exists(), "Output file was not created");

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(output_content.contains("Integration Test"));
    assert!(output_content.contains("let mut vstack_1 = VStack::new();"));

    // Cleanup
    let _ = fs::remove_dir_all(tmp_dir);
}
