# Tick studio

# What is tick studio

Tick studio is a tool for scanning Minecraft videos for moments where the hunger bar icons are shaking and using those jolts to extract internal Minecraft GUI tick time. From this you can infer how long the Minecraft instance has been running for which enables deeper analysis of Minecraft videos.

Using this tick information you can for example:

- detect cuts
- see how much time passed during said cut
- see when the game was relaunched

This information is useful for:

- speedrun moderation
- debunking fake videos
- fun _behind the scenes_ look into the video regarding time

If you want to get help, suggest new features or report bugs join [my discord](https://discord.gg/ubYPyHw2rj)

## How does it work

At the start of GUI rendering during heart icons rendering Java Random instance gets seeded with `tick * 312871` because of that, random numbers generated during the GUI rendering depend on the current tick. When the player is bellow 4 hp (2 hearts) the heart icons start shaking each doing one `nextInt(2)` call, similarly when out of saturation the hunger bar icons shake as well each doing `nextint(3)` instead. Because of that we can simply observe a hunger shake pattern and brute-force every tick, looking for the ticks that could produce that pattern.

All that’s left to do is simply go through the video frame by frame scanning for moments where the hunger is shaking and brute-forcing the possible ticks that shake had to happen on.

## Dream manhunts

This tool was originally a CLI created specifically to scan Dream “uncut” manhunt videos. Later rewritten with `egui` library to make it more user friendly and complete.

Using this tool we were able to find 46 different cuts in videos Dream claimed to be unedited. The scan data and cuts are all documented [here](https://dartjs.com/joltcracker/dream)

## Planned features

Here are some features I planned but couldn't implement in time. Most of them will be most likely implemented in following week or two.

- Manual
  - ability to restore constraints from history
  - copy paste functionality for the constraints
- Scanner
  - better way of selecting the bounding box
  - gui size presets
  - resource pack support
- Explorer
  - more tools for manual cleanup
- Other
  - make tick studio usable without ffmpeg and yt-dlp by disabling features that require it
  - WASM powered website with only manual and explorer tab

# Installation guide

As of now the app requires both `ffmpeg` and `yt-dlp` installed (see planned features).

Go to [Gitlab Releases](https://gitlab.com/fanda8571/tick_studio/-/releases), download and run the latest version.

## Building from source

You can also build the app yourself from source by [getting rust](https://rust-lang.org/learn/get-started/) and running `cargo b -r` .

# Usage

## Manual

This is the quickest and simplest way to crack a single jolt as it doesn’t require video download.

To crack the time simply observe the jolt from the video, frame stepping forward and back often helps to spot the exact pattern.

After that input the pattern into the program by using the arrows, set the food level and make sure to check the low health toggle if the player is bellow 4 hp (2 hearts).

After cracking you should see the result(s). If no result is found try double-checking your input or increasing the maximum time. If you get multiple results and are not sure which is the right one, find another close jolt in the video and crack it as well. The previous one will be in the history sidebar and the cracked time from the second one should be close to it.

## Downloader

Tick studio contains simple utility for downloading videos using `yt-dlp` under the hood. Simply paste the YouTube link and pick the format you want to download.

Depending on your internet speed you should pick either the highest available or at least full-HD. Sometimes 720p videos are AI-upscaled to 1080p by Youtube, from my testing those seem to work fine as well.

After picking the format and output folder, simply click download and the video will download into the target folder.

## Scanner

First select the local video file you wish to scan, then you need to determine the bounding box of the hunger bar during a jolt.

Simplest way of doing this is finding a frame where jolt happened and using paint or similar software getting the box dimensions. The box should be the smallest rectangle that fits the whole hunger bar inside even when jolt is happening.

Then press scan. You’ll see the cropped hotbar at each detection, the scan progress and a stream of all of the new jolt codes and uptimes flooding in.

## Explorer

After importing scanned data, this page will present an XY scatter plot of the data with video time along the X axis and cracked uptime (first match only) on the Y axis. You can click on each jolt to see its information like video time, uptime and jolt code.

After scanning a video, there will be jolts which are scanned incorrectly due to interference from compression artifacts, as well as jolts with multiple matches where the first one is the wrong match. The explorer tab comes with an auto-repair button which automatically detects the linear runs within the jolts and removes the outliers, taking into account jolts with multiple possibilities, collapsing them to the jolt that fits the detected line where possible. You can tune how aggressive this cleanup is using the auto cleanup settings.

While selecting a single points shows it's information, selecting 2 points shows both video and mc time difference between the two, as well as how much time is missing (cut out).

Manual cleanup is also possible by selecting any number of points (either individually or with box selection) and pressing the delete key.
