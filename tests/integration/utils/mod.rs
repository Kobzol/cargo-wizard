use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rexpect::session::{spawn_command, PtySession};
use tempfile::TempDir;

pub struct CargoProject {
    _name: String,
    pub dir: PathBuf,
    _tempdir: TempDir,
    check_on_drop: bool,
}

impl CargoProject {
    pub fn cmd(&self, args: &[&str]) -> Cmd {
        Cmd::default()
            .cwd(&self.dir)
            .args(&["cargo", "wizard", "--colors", "never"])
            .args(args)
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

    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path(path.as_ref()).is_file()
    }

    pub fn read<P: AsRef<Path>>(&self, path: P) -> String {
        let path = path.as_ref();
        std::fs::read_to_string(self.path(path))
            .unwrap_or_else(|e| panic!("Cannot read path {}: {e:?}", path.display()))
    }

    pub fn manifest(&mut self, contents: &str) -> &mut Self {
        self.file("Cargo.toml", contents)
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.path("Cargo.toml")
    }

    pub fn read_manifest(&self) -> String {
        let path = self.manifest_path();
        self.read(path)
    }

    pub fn config(&mut self, contents: &str) -> &mut Self {
        self.file(self.config_path(), contents)
    }

    pub fn config_path(&self) -> PathBuf {
        self.path(".cargo/config.toml")
    }

    pub fn read_config(&self) -> String {
        let path = self.config_path();
        self.read(path)
    }

    pub fn disable_check_on_drop(mut self) -> Self {
        self.check_on_drop = false;
        self
    }
}

impl Drop for CargoProject {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Do not delete the directory if an error has occurred
            let path = std::mem::replace(&mut self._tempdir, TempDir::new().unwrap()).into_path();
            eprintln!("Directory of failed test located at: {}", path.display());
        } else if self.check_on_drop {
            Cmd::default()
                .args(&["cargo", "check"])
                .cwd(&self.dir)
                .run()
                .expect("Cannot run cargo check on the test project")
                .assert_ok();
        }
    }
}

#[derive(Default)]
pub struct Cmd {
    arguments: Vec<String>,
    cwd: Option<PathBuf>,
    stdin: Vec<u8>,
}

impl Cmd {
    pub fn start_terminal(self) -> anyhow::Result<Terminal> {
        let session = spawn_command(self.create_std_cmd(false), Some(1000))?;
        Ok(Terminal { session })
    }

    pub fn run(self) -> anyhow::Result<Output> {
        let mut command = self.create_std_cmd(true);

        let mut child = command.spawn()?;
        {
            let mut child_stdin = child.stdin.take().unwrap();
            child_stdin.write_all(&self.stdin)?;
        }

        let output = child.wait_with_output()?;
        Ok(output)
    }

    fn create_std_cmd(&self, capture_output: bool) -> Command {
        let mut command = Command::new(&self.arguments[0]);
        for arg in &self.arguments[1..] {
            command.arg(arg);
        }
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command.stdin(Stdio::piped());
        if capture_output {
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
        }

        let path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{}", debug_target_dir().display(), path);

        command.env("PATH", path);
        command
    }

    pub fn args(mut self, args: &[&str]) -> Self {
        self.arguments.extend(args.iter().map(|s| s.to_string()));
        self
    }

    pub fn cwd(self, cwd: &Path) -> Self {
        Self {
            cwd: Some(cwd.to_path_buf()),
            ..self
        }
    }
}

pub struct Terminal {
    pub session: PtySession,
}

impl Terminal {
    pub fn expect(&mut self, text: &str) -> anyhow::Result<()> {
        self.session.exp_string(text)?;
        Ok(())
    }

    pub fn line(&mut self, text: &str) -> anyhow::Result<()> {
        self.session.send_line(text)?;
        Ok(())
    }

    pub fn key_enter(&mut self) -> anyhow::Result<()> {
        self.session.send_line("")?;
        Ok(())
    }

    pub fn key_down(&mut self) -> anyhow::Result<()> {
        // Arrow down, detected through `showkey -a`
        self.session.send("\x1b\x5b\x42")?;
        self.session.flush()?;
        Ok(())
    }

    /// Find a line that begings by `> {prefix}` by going through a list using the down arrow key.
    pub fn select_line(&mut self, prefix: &str) -> anyhow::Result<()> {
        let max_tries = 20;
        for _ in 0..max_tries {
            if self
                .session
                .exp_regex(&format!("\n>\\s*{prefix}.*"))
                .is_ok()
            {
                return self.key_enter();
            }
            self.key_down()?;
        }
        eprintln!("Could not find line beginning with {prefix} in {max_tries} tries.");
        // Print terminal output
        self.session
            .exp_string(&format!("<missing {prefix} in list>"))?;
        unreachable!();
    }

    pub fn wait(self) -> anyhow::Result<()> {
        self.session.process.wait()?;
        Ok(())
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
        .args(["new", "--bin", name])
        .current_dir(dir.path())
        .status()?;
    assert!(status.success());

    let path = dir.path().join(name);

    println!("Created Cargo project {} at {}", name, path.display());

    let mut project = CargoProject {
        _name: name.to_string(),
        dir: path,
        _tempdir: dir,
        check_on_drop: true,
    };

    // Normalize the manifest to avoid any surprises
    project.manifest(
        r#"[package]
name = "foo"
version = "0.1.0"
edition = "2021"
"#,
    );

    Ok(project)
}

fn debug_target_dir() -> PathBuf {
    let mut target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    target_dir.push("target");
    target_dir.push("debug");
    target_dir
}
