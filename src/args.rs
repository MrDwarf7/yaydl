use clap::Parser;

/// Command line arguments for yaydl.
///
/// Now with ENV Support!
#[derive(Parser, Default, Debug)]
#[clap(version, about = "Yet Another Youtube Down Loader", long_about = None)]
pub struct Args {
    #[clap(long = "only-audio", short = 'x', help = "Only keeps the audio stream")]
    pub onlyaudio: bool,

    #[clap(
        long = "keep-temp-file",
        short = 'k',
        help = "Keeps all downloaded data even with --only-audio"
    )]
    pub keeptempfile: bool,

    #[clap(long, short = 'v', help = "Talks more while the URL is processed")]
    pub verbose: bool,

    #[clap(
        long = "audio-format",
        short = 'f',
        help = "Sets the target audio format (only if --only-audio is used).\nSpecify the file extension here.",
        default_value = "mp3"
    )]
    pub audioformat: String,

    #[clap(long = "output", short = 'o', help = "Sets the output file name")]
    pub outputfile: Option<String>,

    #[clap(
        long,
        help = "The port of your web driver (required for some sites)",
        env = "YAYDL_WEBDRIVER_PORT"
    )]
    pub webdriver: Option<u16>,

    #[clap(help = "Sets the input URL to use", index = 1)]
    pub url: String,

    #[clap(
        long = "invidious-instance",
        short = 'i',
        help = "Sets the Invidious instance to use",
        env = "YAYDL_INVIDIOUS_INSTANCE"
    )]
    pub invidious_instance: Option<String>,
}

impl Args {
    pub fn parse_webdriver(&self) -> u16 {
        self.webdriver.unwrap_or(0)
    }
}
