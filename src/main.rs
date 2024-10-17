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
// - main.rs file -

mod args;
mod definitions;
mod download;
mod ffmpeg;
mod handlers;
mod prelude;
mod processor;

use crate::args::Args;
use crate::prelude::{from_env_proxy, Printer};
use crate::processor::{process_video, VideoProcessingResult};
use anyhow::Result;
use clap::Parser;
use processor::{InputOutputPaths, TargetFile};
use std::borrow::Borrow;

// usage:
// let v = VIDEO{info: String::new(), title:String::new(), mime:String::new()};
// println!("{:#?}",v);
pub struct VIDEO {
    info: String,
    title: String,
    mime: String,
}

// Stop the linter from blowing up because of inventory::collect! macro
#[allow(non_local_definitions)]
fn main() -> Result<()> {
    // Argument parsing:
    let args = Args::parse();

    inventory::collect!(&'static dyn definitions::SiteDefinition);

    let mut video = VIDEO {
        info: String::new(),
        title: String::new(),
        mime: String::new(),
    };

    // Used in loop below:
    let mut printer = Printer::new();
    let in_url = &args.url;
    let mut site_def_found = false;

    for handler in inventory::iter::<&dyn definitions::SiteDefinition> {
        // Find a known handler for <in_url>:
        if !handler.can_handle_url(in_url) {
            continue;
        }

        // This one is it.
        site_def_found = true;
        printer
            .lock()
            .add(format!("Fetching from {}.", handler.display_name()))
            .flush();

        if handler.web_driver_required() && args.parse_webdriver() == 0 {
            // This handler would need a web driver, but none is supplied to yaydl.
            printer.web_driver_req(handler.display_name());
            continue;
        }

        let processing_result = process_video(
            handler.borrow(),
            &args,
            &mut video,
            in_url,
            args.parse_webdriver(),
        )?;

        check_result(processing_result, &args, &mut printer)?;

        // match processing_result {
        //     VideoProcessingResult::VideoNotFound => {
        //         println!("The video could not be found. Invalid link?");
        //     }
        //     VideoProcessingResult::TitleNotFound => {
        //         println!("The video title could not be extracted. Invalid link?");
        //     }
        //     VideoProcessingResult::ReadyToProcess(ready) => {
        //         if args.verbose {
        //             println!("Starting the download.");
        //         }
        //
        //         let url = ready.url.clone();
        //         let targetfile = TargetFile::from(ready);
        //
        //         match targetfile.force_ffmpeg {
        //             true => {
        //                 download::download_from_playlist(
        //                     &url,
        //                     &targetfile.target_filename,
        //                     args.verbose,
        //                 )?;
        //             }
        //             false => {
        //                 download::download(&url, &targetfile.target_filename)?;
        //             }
        //         }
        //
        //         if (args.onlyaudio && targetfile.target_ext != *args.audioformat)
        //             || targetfile.force_ffmpeg
        //         {
        //             if args.verbose {
        //                 println!("Post-processing.");
        //             }
        //
        //             let mut paths_for = InputOutputPaths::from(&targetfile);
        //
        //             paths_for.to_audio_mut(args.onlyaudio, args.audioformat);
        //
        //             if !args.keeptempfile {
        //                 std::fs::remove_file(&targetfile.target_filename)?;
        //             }
        //
        //             printer
        //                 .add(format!(
        //                     "\"{}\" successfully downloaded.",
        //                     paths_for.output_to_string()
        //                 ))
        //                 .flush();
        //         } else {
        //             printer
        //                 .add(format!(
        //                     "\"{}\" successfully downloaded.",
        //                     &targetfile.target_filename
        //                 ))
        //                 .flush();
        //         }
        //     }
        // }

        // Stop looking for other handlers:
        break;
    }

    if !site_def_found {
        println!(
            "yaydl could not find a site definition that would satisfy {}. Exiting.",
            in_url
        );
    }

    Ok(())
}

fn check_result(
    processing_result: VideoProcessingResult,
    args: &Args,
    printer: &mut Printer<String>,
) -> Result<()> {
    match processing_result {
        VideoProcessingResult::VideoNotFound => {
            println!("The video could not be found. Invalid link?");
        }
        VideoProcessingResult::TitleNotFound => {
            println!("The video title could not be extracted. Invalid link?");
        }
        VideoProcessingResult::ReadyToProcess(ready) => {
            if args.verbose {
                println!("Starting the download.");
            }

            let url = ready.url.clone();
            let targetfile = TargetFile::from(ready);

            match targetfile.force_ffmpeg {
                true => {
                    targetfile.download_from_playlist(&url, args.verbose)?;
                }
                false => {
                    targetfile.download(&url)?;
                }
            }

            if (args.onlyaudio && targetfile.target_ext != *args.audioformat)
                || targetfile.force_ffmpeg
            {
                if args.verbose {
                    println!("Post-processing.");
                }

                let mut paths_for = InputOutputPaths::from(&targetfile);

                paths_for.to_audio_mut(args.onlyaudio, args.audioformat.clone());

                if !args.keeptempfile {
                    std::fs::remove_file(&targetfile.target_filename)?;
                }

                printer
                    .add(format!(
                        "\"{}\" successfully downloaded.",
                        paths_for.output_to_string()
                    ))
                    .flush();
            } else {
                printer
                    .add(format!(
                        "\"{}\" successfully downloaded.",
                        &targetfile.target_filename
                    ))
                    .flush();
            }
        }
    }

    Ok(())
}
