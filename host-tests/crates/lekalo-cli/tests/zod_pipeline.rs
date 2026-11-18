use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const REFERENCE_PROJECT: &str = "tests/fixtures/orchestration/project";
fn workspace_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(relative)
}

/// GitHub's Windows runners export `%TEMP%` spelled with the 8.3 alias of
/// the profile directory; the selection policy denies alias spellings.
fn alias_free_path(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().expect("fixture path must exist");
    #[cfg(windows)]
    match canonical.to_string_lossy().strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => canonical,
    }
    #[cfg(not(windows))]
    canonical
}

fn lekalo_with(dir: &Path, head: &[&str], adapter_args: &[&str]) -> Output {
    let mut args: Vec<&str> = head.to_vec();
    args.push("--");
    args.extend_from_slice(adapter_args);
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(&args)
        .current_dir(alias_free_path(dir))
        .output()
        .expect("run the real lekalo binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn exit_code(output: &Output) -> u8 {
    output.status.code().expect("exit code") as u8
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf8")
}

fn copy_dir(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).expect("create target dir");
    for entry in std::fs::read_dir(source).expect("read source") {
        let entry = entry.expect("read entry");
        let target_path = target.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target_path);
        } else {
            std::fs::copy(entry.path(), target_path).expect("copy file");
        }
    }
}

static NEXT_ROOT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Run a body against a fresh copy of the orchestration fixture project,
/// with the child working directory set to the copy so the adapter
/// vector resolves project-relative, exactly like an operator run.
fn with_project(body: impl FnOnce(&Path)) {
    let id = NEXT_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        + u64::from(std::process::id());
    let root = workspace_path("target")
        .join("orchestration-cli-tests")
        .join(format!("case-{id}"));
    copy_dir(&workspace_path(REFERENCE_PROJECT), &root);
    body(&root);
    let _ = std::fs::remove_dir_all(&root);
}

const ADAPTER_ARGS: &[&str] = &["node", "adapters/node-typescript/node-adapter.mjs"];
fn lekalo_in(dir: &Path, head: &[&str], adapter: bool) -> Output {
    let mut args: Vec<&str> = head.to_vec();
    if adapter {
        args.push("--");
        args.extend_from_slice(ADAPTER_ARGS);
    }
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(&args)
        .current_dir(alias_free_path(dir))
        .output()
        .expect("run the real lekalo binary")
}

/// Run the real binary with one custom adapter vector after `--`.
/// The compact launch profile handed to the adapter through argv: the
/// read roots cover the canonical IR evidence home, the only input
/// generation consumes. Never an ambient grant.
const ZOD_E2E_PROFILE: &str = concat!(
    r#"{"id":"generate","mode":"observed","target":"node-typescript",""#,
    r#"readRoots":[{"path":".lekalo/cache/ir","kind":"tree"}],"#,
    r#""exclusions":[],"provenance":{"origin":"declared","#,
    r#""revision":"issue-45-e2e-0001","disposition":"public-fixture"}}"#
);

/// Copy the committed generation artifact into the project copy so the
/// lock can pin it exactly like the reference implementation.
fn with_zod_adapter(root: &Path) {
    let home = root.join("adapters/node-typescript");
    std::fs::create_dir_all(&home).expect("adapter home");
    std::fs::copy(
        workspace_path("tests/external/lekalo-typescript/adapters/node-typescript/adapter-zod.mjs"),
        home.join("adapter-zod.mjs"),
    )
    .expect("copy the generation artifact");
}

/// Run the real binary with the generation artifact vector.
fn zod_lekalo_in(root: &Path, head: &[&str]) -> Output {
    let mut args: Vec<&str> = head.to_vec();
    args.push("--");
    args.extend_from_slice(&[
        "node",
        "adapters/node-typescript/adapter-zod.mjs",
        "--lekalo-project-profile-json",
        ZOD_E2E_PROFILE,
    ]);
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(&args)
        .current_dir(alias_free_path(root))
        .output()
        .expect("run the real lekalo binary")
}

#[test]
fn the_zod_generation_artifact_drives_the_real_pipeline() {
    with_project(|root| {
        with_zod_adapter(root);
        let lock = zod_lekalo_in(root, &["--json", "lock"]);
        assert_eq!(exit_code(&lock), 0, "stdout={}", stdout(&lock));
        // Dry run: the plan lists the whole generated set, nothing exists.
        let planned = zod_lekalo_in(
            root,
            &[
                "--json",
                "generate",
                "--target",
                "node-typescript",
                "--dry-run",
            ],
        );
        assert_eq!(exit_code(&planned), 0, "stdout={}", stdout(&planned));
        let receipt: serde_json::Value = serde_json::from_str(&stdout(&planned)).expect("json");
        let writes = receipt["targets"][0]["writes"].as_array().expect("writes");
        assert!(
            writes.len() >= 4,
            "every generated file is planned: {writes:?}"
        );
        let paths: Vec<&str> = writes
            .iter()
            .map(|write| write["path"].as_str().expect("path"))
            .collect();
        assert!(paths.contains(&"src/generated/node-typescript/zod/notify.ts"));
        assert!(paths.contains(&"src/generated/node-typescript/zod/runtime.ts"));
        assert!(paths.contains(&"src/generated/node-typescript/zod/index.ts"));
        assert!(paths.contains(&"src/generated/node-typescript/zod/notify.map.json"));
        assert!(!root
            .join("src/generated/node-typescript/zod/notify.ts")
            .exists());
        // Apply: the files exist, the manifest attributes schema and data
        // kinds, and the sidecar declaration ranges land as source maps
        // bound to the module artifacts.
        let applied = zod_lekalo_in(root, &["--json", "generate", "--target", "node-typescript"]);
        assert_eq!(exit_code(&applied), 0, "stdout={}", stdout(&applied));
        let applied_receipt: serde_json::Value =
            serde_json::from_str(&stdout(&applied)).expect("json");
        assert_eq!(applied_receipt["targets"][0]["state"], "applied");
        assert!(root
            .join("src/generated/node-typescript/zod/notify.ts")
            .exists());
        let manifest_text =
            std::fs::read_to_string(root.join(".lekalo/generated/manifests/ownership.json"))
                .expect("manifest exists");
        assert!(manifest_text.contains("\"schema\""), "kind schema recorded");
        assert!(manifest_text.contains("\"data\""), "kind data recorded");
        assert!(
            manifest_text.contains("source_maps"),
            "source maps recorded"
        );
        assert!(
            manifest_text.contains("notify.ts"),
            "source maps bind module artifacts"
        );
        // The drift gate accepts the fresh generation.
        let check = lekalo_in(root, &["generate", "--check"], false);
        assert_eq!(exit_code(&check), 0, "stdout={}", stdout(&check));
        assert!(stdout(&check).contains("clean 4"));
        // Tamper: the gate refuses.
        let generated = root.join("src/generated/node-typescript/zod/notify.ts");
        let body = std::fs::read_to_string(&generated).expect("read generated");
        std::fs::write(
            &generated,
            body.replace("NotifyChannelSchema", "NotifyChannelSchemaTampered"),
        )
        .expect("tamper");
        let drifted = lekalo_in(root, &["generate", "--check"], false);
        assert_eq!(exit_code(&drifted), 1, "stdout={}", stdout(&drifted));
    });
}
