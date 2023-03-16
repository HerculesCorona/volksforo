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
