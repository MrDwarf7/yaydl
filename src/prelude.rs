use std::io::Write;

/// Portal Printer to communicate with the user.
pub struct Printer<S>
where
    S: AsRef<str>,
{
    buf: S,
    stdout: std::io::Stdout,
}

impl Printer<String> {
    pub fn new() -> Self {
        let mut s = Self {
            buf: String::new(),
            stdout: std::io::stdout(),
        };
        s.lock();
        s
    }

    pub fn lock(&mut self) -> &mut Self {
        match self.stdout.lock().write_all(b"\x1b[?25l").as_ref() {
            Ok(_) => self,
            Err(_) => self,
        }
    }

    pub fn add(&mut self, s: impl AsRef<str>) -> &mut Self {
        self.buf.push_str(s.as_ref());
        self
    }

    pub fn flush(&mut self) -> &mut Self {
        self.stdout.write_all(self.buf.as_bytes()).unwrap();
        self.buf.clear();
        self
    }
}

impl Printer<String> {
    pub fn web_driver_req<H>(&mut self, handler: H)
    where
        H: Into<String> + AsRef<str>,
    {
        self.add(format!("{} requires a web driver installed and running as described in the README. Please tell yaydl which port to use (yaydl --webdriver <PORT>) and try again.", handler.as_ref())).flush();
    }
}

pub fn from_env_proxy(url: impl AsRef<str>) -> Option<ureq::Agent> {
    if let Some(env_proxy) = env_proxy::for_url(&url::Url::parse(url.as_ref()).unwrap()).host_port()
    {
        // Use a proxy:
        let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
        let agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        Some(agent)
    } else {
        None
    }
}
