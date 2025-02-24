pub trait ATCommand {
    fn command(&self) -> String;
    fn parse_response(&self, response: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct BasicCommand {
    cmd: String,
}

impl BasicCommand {
    pub fn new(cmd: &str) -> Self {
        Self { cmd: cmd.to_string() }
    }
}

impl ATCommand for BasicCommand {
    fn command(&self) -> String {
        self.cmd.clone()
    }

    fn parse_response(&self, response: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(response.to_string())
    }
}
