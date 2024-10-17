/*
 * The contents of this file are subject to the terms of the
 * Common Development and Distribution License, Version 1.0 only
 * (the "License").  You may not use this file except in compliance
 * with the License.
 *
 * See the file LICENSE in this distribution for details.
 * A copy of the CDDL is also available via the Internet at
 * http://www.opensource.org/licenses/cddl1.txt
 *
 * When distributing Covered Code, include this CDDL HEADER in each
 * file and include the contents of the LICENSE file from this
 * distribution.
 */

// Yet Another Youtube Down Loader
// - YouTube and Invidious handler -

use crate::definitions::SiteDefinition;
use crate::VIDEO;

use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use std::{
    env,
    sync::{LazyLock, RwLock},
};

// Starting with yaydl 0.13.0, this handler uses Invidious instead
// of YouTube. In no way am I interested in playing cat and mouse
// against Google.

// The environment variable YAYDL_INVIDIOUS_INSTANCE can be used to
// define the instance to use, otherwise, yaydl defaults to this:
const INVIDIOUS_INSTANCE: &str = "https://invidious.privacyredirect.com";

static ID_REGEX: LazyLock<RwLock<Regex>> = LazyLock::new(|| {
    std::sync::RwLock::new(Regex::new(r"(?:v=|\.be/|shorts/)(.*?)(&.*)*$").unwrap())
});

fn get_invidious_instance() -> String {
    let invidious_env = env::var("YAYDL_INVIDIOUS_INSTANCE");
    if invidious_env.is_ok() {
        invidious_env.unwrap_or(INVIDIOUS_INSTANCE.to_string())
    } else {
        INVIDIOUS_INSTANCE.to_string()
    }
}

// PERF: This could just be an Into<String> or Into<Str> or Into<Url> or AsRef<str> or something.
// and replace the usage of the function with video.info.as_str() or something.
// and you don't have a large struct being parsed & no need to be mutable either
//
// fn get_video_info(video: impl Into<String> , url: &str) -> Result<Html> {
// let video = video.into();
// if video.is_empty() {
//
// etc..
//
fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Html> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        // Exchange the URL -> Invidious:
        let id = ID_REGEX
            .read()
            .unwrap()
            .captures(url)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        // let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();

        let local_url = format!("{}/watch?v={}", get_invidious_instance(), id).to_owned();

        // Initialize the agent:
        // let mut agent = ureq::agent();
        // if let Some(env_proxy) = env_proxy::for_url(&Url::parse(&local_url)?).host_port() {
        //     // if let Some(env_proxy) = env_proxy::for_url(&url_p).host_port() {
        //     // Use a proxy:
        //     let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
        //     agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        // }

        video.info.push_str(
            crate::from_env_proxy(&local_url)
                .unwrap_or(ureq::agent())
                .get(&local_url)
                .call()?
                .into_string()?
                .as_str(),
        );
    }
    Ok(Html::parse_document(&video.info))
}

// pub fn from_env_proxy(url: impl AsRef<str>) -> Option<ureq::Agent> {
//     if let Some(env_proxy) = env_proxy::for_url(&Url::parse(url.as_ref()).unwrap()).host_port() {
//         // Use a proxy:
//         let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
//         let agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
//         Some(agent)
//     } else {
//         None
//     }
// }

// Implement the site definition:
struct YouTubeHandler;
impl SiteDefinition for YouTubeHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"invidious\.|(?:www\.)?youtu(?:be\.com|\.be)/")
            .unwrap()
            .is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // Left as an exercise to the user. TBD.
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let video_info = get_video_info(video, url)?;

        let title_selector = Selector::parse(r#"meta[property="og:title"]"#).unwrap();
        let title_elem = video_info.select(&title_selector).next().unwrap();
        let title_contents = title_elem.value().attr("content").unwrap();

        Ok(title_contents.to_string())
    }

    fn find_video_direct_url<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        let video_info = get_video_info(video, url)?;

        let mut url_to_choose = "".to_string();

        // Find the least horrible format:
        let quality_selector = Selector::parse(r#"source"#).unwrap();
        let quality_iter = video_info.select(&quality_selector);

        // Please - we have language tools to not have to do these things in Rust;
        // mutability is (esp when a single variable and very generally) a code smell.
        let mut last_vq: String = String::new();

        for source in quality_iter {
            // The highest quality wins.
            let this_tag = source;
            let this_vq = this_tag.value().attr("label").unwrap_or("");

            let is_same_or_better_video = this_vq != last_vq && last_vq != "medium";

            // Try to download the best file.
            if is_same_or_better_video {
                let this_mimetype = this_tag.value().attr("type").unwrap();

                // Example: type="video/mp4; codecs=&quot;avc1.64001F, mp4a.40.2&quot;"
                // Fetch the video/mp4 substring:
                let mut mime_split = this_mimetype.split(";");
                video.mime = mime_split.next().unwrap().to_string();

                let relative_url = this_tag.value().attr("src").unwrap();
                url_to_choose = format!("{}{}", get_invidious_instance(), relative_url);

                // Only update last_vq if it's the best format yet.
                last_vq = String::from(this_vq);
            }
        }

        if url_to_choose.is_empty() {
            Err(anyhow::Error::msg(
                "Could not find a working video - aborting.".to_string(),
            ))
        } else {
            Ok(url_to_choose)
        }
    }

    fn does_video_exist<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        let _video_info = get_video_info(video, url);
        Ok(!video.info.is_empty())
    }

    fn display_name(&self) -> String {
        "Invidious".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        video: &'a mut VIDEO,
        _url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        // By this point, we have already filled VIDEO_MIME. Let's just use that.
        let mut ext = "mp4";
        if video.mime.contains("/webm") {
            ext = "webm";
        } else if video.mime.contains("audio/mp4") {
            ext = "m4a";
        }

        Ok(ext.to_string())
    }

    fn web_driver_required(&self) -> bool {
        false
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &YouTubeHandler as &dyn SiteDefinition
}
