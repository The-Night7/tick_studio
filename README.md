# Tick Studio: automatic Arch Linux builds

Copy `.github/workflows/arch-package.yml` and the entire `packaging/aur/` folder into the root of https://github.com/The-Night7/tick_studio, then commit and push.

The workflow runs for every branch push and can also be triggered manually in **Actions → Build Arch Linux package → Run workflow**. It runs `makepkg` as an unprivileged user within an up-to-date Arch Linux container, builds the **exact pushed commit**, and uploads the `.pkg.tar.zst` artifact for 30 days.

## Download and install

GitHub → Actions → Build Arch Linux package → successful run → Artifacts → download `tick-studio-arch-x86_64-N`. Unzip and install on Arch Linux:

    sudo pacman -U ./tick-studio-git-*.pkg.tar.zst
    tick-studio

This artifact is an **Arch Linux binary package**, not an AUR submission or GitHub Release. It supports x86_64 only.

## Optional manual build

On Arch Linux:

    sudo pacman -S --needed base-devel git rust cargo ffmpeg yt-dlp
    cd packaging/aur
    makepkg -si

## Known limitations

- The project has not been compiled in this environment; a build in GitHub Actions is the actual validation.
- Your earlier log showed undefined symbols in `ring` at the linking step. LTO is disabled here to reduce one possible source of native-linking incompatibility. This is not a guaranteed fix; a failure still appears in the Actions log rather than being silently ignored.
- The package currently uses `license=('unknown')` because the upstream repository has no visible license; add the correct license after choosing one.
- This workflow deliberately does not publish to the AUR or GitHub Releases. It uploads artifacts per run.
- Updating the Arch container may introduce breaking dependencies; consider pinning a container digest after validating the build.
