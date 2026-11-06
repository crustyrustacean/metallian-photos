# metallian-photos

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./License.txt)
[![Rust 2024 Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

A photo storage API which backs the [Crusty Metallian](https://crusty-metallian.net) music blog, using Actix Web, SQLite, and Cloudflare R2.

## Photo operation recovery

Uploads and deletions are tracked in SQLite. Only `ready` photos appear in galleries or image endpoints. On startup and every five minutes while running, the application cleans up interrupted uploads and retries unfinished deletes. Recovery failures are logged and retried; a failed upload must be submitted again. A deletion that returns an error may still finish during recovery. Multi-file uploads are independent: earlier files remain if a later file fails.

Recovery coordinates operations within a single application process. Do not run multiple processes against the same SQLite database and storage without adding cross-process coordination. SQLite and R2 are not a single transaction: an unusually late storage write after an ambiguous failure could still leave an orphaned object. The local memory storage backend is ephemeral and does not retain image bytes across restarts.
