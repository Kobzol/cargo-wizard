use rexpect::session::PtySession;

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
