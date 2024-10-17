use std::path::PathBuf;

use crate::{definitions, Args};
use anyhow::Result;

pub struct ReadyToProcess {
    pub title: String,
    pub url: String,
    pub ext: String,
    pub is_playlist: bool,
}

pub enum VideoProcessingResult {
    VideoNotFound,
    TitleNotFound,
    ReadyToProcess(ReadyToProcess),
}

pub fn process_video(
    handler: &dyn definitions::SiteDefinition,
    args: &Args,
    video: &mut crate::VIDEO,
    in_url: &str,
    webdriverport: u16,
) -> Result<VideoProcessingResult> {
    if !handler.does_video_exist(video, in_url, webdriverport)? {
        return Ok(VideoProcessingResult::VideoNotFound);
    }

    if args.verbose {
        println!("The requested video was found. Processing...");
    }

    let vt = match handler.find_video_title(video, in_url, webdriverport) {
        Err(_e) => "".to_string(),
        Ok(title) => title,
    };

    if vt.is_empty() {
        return Ok(VideoProcessingResult::TitleNotFound);
    }

    if args.verbose {
        println!("Title: {}", vt);
    }

    Ok(VideoProcessingResult::ReadyToProcess(ReadyToProcess {
        title: vt,
        url: handler.find_video_direct_url(video, in_url, webdriverport, args.onlyaudio)?,
        ext: handler.find_video_file_extension(video, in_url, webdriverport, args.onlyaudio)?,
        is_playlist: handler.is_playlist(in_url, webdriverport).unwrap_or(false),
    }))
}

impl From<VideoProcessingResult> for ReadyToProcess {
    fn from(vpr: VideoProcessingResult) -> Self {
        match vpr {
            VideoProcessingResult::ReadyToProcess(rtp) => rtp,
            _ => panic!("Expected ReadyToProcess"),
        }
    }
}

pub struct TargetFile {
    // pub target_title: String,
    // This should 1000% be an enum of types you're okay with supporting.
    pub target_ext: String,
    pub target_filename: String,
    pub force_ffmpeg: bool,
}

impl From<ReadyToProcess> for TargetFile {
    fn from(rtp: ReadyToProcess) -> Self {
        let target_filename = format!(
            "{}.{}",
            rtp.title.trim().replace(
                &['|', '\'', '\"', ':', '\'', '\\', '/', '?', '*'][..],
                r#""#
            ),
            rtp.ext
        )
        .to_string();

        match rtp.is_playlist {
            true => Self {
                // target_title: rtp.title,
                target_ext: rtp.ext,
                target_filename,
                force_ffmpeg: true,
            },
            false => Self {
                // target_title: rtp.title,
                target_ext: rtp.ext,
                target_filename,
                force_ffmpeg: false,
            },
        }
    }
}

impl TargetFile {
    pub fn download_from_playlist(&self, url: impl AsRef<str>, verbosity: bool) -> Result<()> {
        crate::download::download_from_playlist(url.as_ref(), &self.target_filename, verbosity)?;
        Ok(())
    }

    pub fn download(&self, url: impl AsRef<str>) -> Result<()> {
        crate::download::download(url.as_ref(), &self.target_filename)?;
        Ok(())
    }
}

pub struct InputOutputPaths {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl From<&TargetFile> for InputOutputPaths {
    fn from(tf: &TargetFile) -> Self {
        Self {
            input: PathBuf::from(&tf.target_filename),
            output: PathBuf::from(&tf.target_filename),
        }
    }
}

impl InputOutputPaths {
    pub fn set_ext_output(&mut self, ext: impl AsRef<str>) -> &mut Self {
        self.output.set_extension(ext.as_ref());
        self
    }

    pub fn to_audio_mut(&mut self, value: bool, ext: impl AsRef<str>) {
        match value {
            true => {
                // clone the input so we can drop it here righnt after
                let input = self.input.clone();
                crate::ffmpeg::to_audio(&input, self.set_ext_output(ext).output.as_path());
            }
            false => {
                let input = self.input.clone();
                crate::ffmpeg::to_audio(&input, self.set_ext_output("mp4").output.as_path());
            }
        }
    }
}

impl InputOutputPaths {
    pub fn output_to_string(self) -> String {
        self.output.to_string_lossy().to_string()
    }
}
