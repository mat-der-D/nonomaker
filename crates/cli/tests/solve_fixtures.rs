use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn solve_command_matches_expected_outputs_for_supported_solvers() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut inputs = collect_input_fixtures(&fixture_dir);
    inputs.sort();

    assert!(
        !inputs.is_empty(),
        "no solve fixtures found in {}",
        fixture_dir.display()
    );

    for solver_case in solver_cases() {
        for input in &inputs {
            assert_fixture_matches_expected(input, solver_case);
        }
    }
}

fn solver_cases() -> &'static [SolverCase] {
    &[
        SolverCase {
            name: "linear",
            expected_suffix: ".partial.json",
        },
        SolverCase {
            name: "fp1",
            expected_suffix: ".fp1.json",
        },
        SolverCase {
            name: "fp2",
            expected_suffix: ".fp2.json",
        },
        SolverCase {
            name: "backtracking",
            expected_suffix: ".output.json",
        },
        SolverCase {
            name: "sat",
            expected_suffix: ".output.json",
        },
    ]
}

fn collect_input_fixtures(fixture_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .unwrap_or_else(|err| {
            panic!(
                "failed to read fixture dir {}: {err}",
                fixture_dir.display()
            )
        })
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".input.json"))
        })
        .collect()
}

fn assert_fixture_matches_expected(input_path: &Path, solver_case: &SolverCase) {
    let expected_path = fixture_path_with_suffix(input_path, solver_case.expected_suffix);
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    let output = Command::new(env!("CARGO_BIN_EXE_nonomaker-cli"))
        .args(["solve", "--solver", solver_case.name, "--input"])
        .arg(input_path)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to execute {} solver for {}: {err}",
                solver_case.name,
                input_path.display()
            )
        });

    assert!(
        output.status.success(),
        "{} solver failed for {}: {}",
        solver_case.name,
        input_path.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout was not valid UTF-8 for {} with solver {}: {err}",
            input_path.display(),
            solver_case.name
        )
    });

    assert_eq!(
        actual.trim_end(),
        expected.trim_end(),
        "{} solver output mismatch for {}",
        solver_case.name,
        input_path.display()
    );
}

fn fixture_path_with_suffix(input_path: &Path, suffix: &str) -> PathBuf {
    let input_name = input_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| panic!("invalid fixture path: {}", input_path.display()));
    let base = input_name.strip_suffix(".input.json").unwrap_or_else(|| {
        panic!(
            "fixture does not end with .input.json: {}",
            input_path.display()
        )
    });
    input_path.with_file_name(format!("{base}{suffix}"))
}

struct SolverCase {
    name: &'static str,
    expected_suffix: &'static str,
}
