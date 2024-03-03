use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

pub struct CargoProject {
    name: String,
    pub dir: PathBuf,
    _tempdir: TempDir,
}

impl CargoProject {
    pub fn run(&self, args: &[&str]) -> anyhow::Result<Output> {
        self.run_with_input(args, &[])
    }

    pub fn run_with_input(&self, args: &[&str], stdin: &[u8]) -> anyhow::Result<Output> {
        let mut command = Command::new("cargo");
        command.arg("wizard");
        for arg in args {
            command.arg(arg);
        }
        command.current_dir(&self.dir);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{}", debug_target_dir().display(), path);

        command.env("PATH", path);

        let mut child = command.spawn()?;
        {
            let mut child_stdin = child.stdin.take().unwrap();
            child_stdin.write_all(stdin)?;
        }

        let output = child.wait_with_output()?;
        Ok(output)
    }

    pub fn path<P: Into<PathBuf>>(&self, path: P) -> PathBuf {
        let path = path.into();
        self.dir.join(path)
    }

    pub fn file<P: AsRef<Path>>(&mut self, path: P, code: &str) -> &mut Self {
        let path = self.path(path.as_ref());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, code).expect("Could not write project file");
        self
    }

    pub fn read<P: AsRef<Path>>(&mut self, path: P) -> String {
        let path = path.as_ref();
        std::fs::read_to_string(self.path(path))
            .expect(&format!("Cannot read path {}", path.display()))
    }

    pub fn manifest(&mut self, code: &str) -> &mut Self {
        self.file("Cargo.toml", code)
    }

    pub fn manifest_path(&mut self) -> PathBuf {
        self.path("Cargo.toml")
    }

    pub fn get_manifest(&mut self) -> String {
        let path = self.manifest_path();
        self.read(path)
    }
}

impl Drop for CargoProject {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Do not delete the directory if an error has occurred
            let path = std::mem::replace(&mut self._tempdir, TempDir::new().unwrap()).into_path();
            eprintln!("Directory of failed test located at: {}", path.display());
        }
    }
}

pub trait OutputExt {
    fn assert_ok(self) -> Self;
    fn assert_error(self) -> Self;

    fn stdout(&self) -> String;
    fn stderr(&self) -> String;
}

impl OutputExt for Output {
    fn assert_ok(self) -> Self {
        if !self.status.success() {
            eprintln!("Stdout: {}", self.stdout());
            eprintln!("Stderr: {}", self.stderr());
            panic!("Output was not successful");
        }
        self
    }

    fn assert_error(self) -> Self {
        if self.status.success() {
            eprintln!("Stdout: {}", self.stdout());
            eprintln!("Stderr: {}", self.stderr());
            panic!("Output was successful");
        }
        self
    }

    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}

pub fn init_cargo_project() -> anyhow::Result<CargoProject> {
    let dir = tempfile::tempdir()?;

    let name = "foo";
    let status = Command::new("cargo")
        .args(&["new", "--bin", name])
        .current_dir(dir.path())
        .status()?;
    assert!(status.success());

    let path = dir.path().join(name);

    println!("Created Cargo project {} at {}", name, path.display());

    Ok(CargoProject {
        name: name.to_string(),
        dir: path,
        _tempdir: dir,
    })
}

fn debug_target_dir() -> PathBuf {
    let mut target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    target_dir.push("target");
    target_dir.push("debug");
    target_dir
}
