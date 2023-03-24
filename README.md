# Volksforo
A traditional web forum built in Rust with modern technology to be fast, secure, scalable, and stable.

## Stack
 - Rust
   - actix-web
   - askama
 - ScyllaDB
 - S3
 - NPM
   - Vanilla JS
   - SCSS for stylesheets
   - SWC for asset compilation

## Aspirations
 - Minimal bloat.
 - Compatability with JS-less Tor.
 - Unit test coverage.
 - Event driven WebSocket subscriptions.
 - Total replacement for XenForo.

## Environment
 - Example `.env` file
 - ScyllaDB
   + Required. Database agnosticism not planned.
 - S3 Storage
   + Any S3-compatible storage API for attachments.
   + Suggested to use [MinIO](https://min.io/) (FOSS + Self-Hosted)
 - FFMPEG
   + Linuxchads may simply install ffmpeg through their package manager and drink a lemonade.
   + Windows users need to set up bindings for Clang and FFMPEG. Try the following:
     1. Follow the [Rust for Windows guide](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup).
     2. Install the [MSVC Rust toolchain](https://rust-lang.github.io/rustup/installation/windows.html).
     3. Install via [vcpkg](https://github.com/microsoft/vcpkg) the [ffmpeg](https://trac.ffmpeg.org/wiki/CompilationGuide/vcpkg) and [LLVM](https://learn.microsoft.com/en-us/vcpkg/users/examples/selecting-llvm-features). Be sure to install use the right vcpkg triplets (probably the x64 variants, which are NOT default on x64 systems!).
     4. Completely close down VS Code, PowerShell and other command interfaces, and run `cargo clean`.
     5. Run `ls env:` in PS and ensure that `LIBCLANG_PATH` is correctly set to the _directory_ that contains `libclang.dll`. For me, this was `C:\dev\vcpkg\installed\x64-windows-static-md\bin`.
     6. Say three Hail Marys, two Our Fathers, and run `cargo build`.
 - node and webpack
   + Install [npm](https://nodejs.org/en/download/).
   + Run `npm install` from the root directory to install node dependencies.
   + Run `npx webpack` from the root directory to deploy browser-friendly resource files.
   + _webpack will be replaced with SWC when SASS compilation is available._

### WebM Validation Notes
 - https://www.webmproject.org/docs/container/
 - VP8
 - VP9
 - AV1
 - OPUS
 - VORBIS

## Contributions
### Code Guidelines
 - We use [rustfmt](https://github.com/rust-lang/rustfmt).
 - `cargo clippy` whenever possible.
 - Try to eliminate warnings.

### Database Guidelines
 - Any data which would apply to two types of content (i.e. posts, chat messages, profile posts) should interact with the `ugc` tables, not individual content type tables.
 - Usernames should be referenced by `user_id,created_at DESC` from `user_name`. User rows can be deleted, but a historical reference for their name will be added to this table. This complies with [GDPR software requirements](https://gdpr.eu/right-to-be-forgotten).
