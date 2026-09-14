Now I have the complete context. Let me also check main.rs to see how `session` is used in `load`, and check the current `Pick::new` (post-edit):

From the earlier edit:
```rust
    fn new(prompt: Option<&str>, session: &Path) -> (Self, Task<Message>) {
        let project = env::current_dir().unwrap_or_default();
        let session = session.to_path_buf();

        let mut pick = Self {
            ...
            session: session.clone(),
            ...
        };

        let load = Task::perform(Session::load(session), Message::SessionLoaded);
```

And main():
```rust
    let session = match resume_path {
        Some(path) => resolve_session(&project, &path),
        None if resume => Session::latest_path(&project)
            .unwrap_or_else(|| Session::fresh_path(&project)),
        None => Session::fresh_path(&project),
    };

    iced::application(
        move || Pick::new(prompt.as_deref(), &session),
        ...
```

And `resolve_session` is a free function in main.rs that returns a PathBuf.

OK, the refactor plan:

**core/src/session.rs:**

1. Add a `File` type before `struct Session`:

```rust
/// A file within a project's session directory: an opaque
/// handle to one of the `.pick/sessions/*.jsonl` files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct File(PathBuf);

impl File {
    /// The directory in which the project's sessions live.
    fn sessions_dir(project: &Path) -> PathBuf {
        project.join(".pick/sessions")
    }

    /// The file of a new session, named by its start:
    /// `YYYYMMDD-HHMMSS` in the local time, with the sub-second
    /// digits appended when a session already holds the name, so
    /// the names stay chronological under plain string order.
    pub fn fresh(project: &Path) -> Self {
        let dir = Self::sessions_dir(project);

        let now = jiff::Timestamp::now();
        let now = now.to_zoned(jiff::tz::TimeZone::system());
        let name = now.strftime("%Y%m%d-%H%M%S").to_string();

        let mut file = format!("{name}.jsonl");

        // A session started in the same second (or same
        // millisecond) appends its sub-second digits — which sort
        // after the extension dot — so `latest` can still tell
        // them apart.
        if dir.join(&file).exists() {
            file = format!("{name}{:03}.jsonl", now.millisecond());
        }
        if dir.join(&file).exists() {
            file = format!(
                "{name}{:03}{:03}.jsonl",
                now.millisecond(),
                now.nanosecond()
            );
        }

        Self(dir.join(file))
    }

    /// The newest of the project's sessions, if there are any:
    /// the greatest name, which is the newest by construction.
    pub fn latest(project: &Path) -> Option<Self> {
        let dir = Self::sessions_dir(project);

        std::fs::read_dir(&dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
            .max()
            .map(Self)
    }

    /// Resolves `name` to one of the project's existing sessions:
    /// as given, relative to the project, or by name in
    /// `.pick/sessions`, with or without the extension.
    pub fn existing(project: &Path, name: &str) -> Option<Self> {
        let dir = Self::sessions_dir(project);

        let candidates = [
            PathBuf::from(name),
            project.join(name),
            dir.join(name),
            dir.join(format!("{name}.jsonl")),
        ];

        candidates
            .iter()
            .find(|candidate| candidate.is_file())
            .map(|candidate| Self(candidate.clone()))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}
```

Wait — should `sessions_dir` be pub or private? main.rs no longer needs it (existing() absorbs it). Tests use `Session::sessions_dir` — the tests are in the same module and can call private `File::sessions_dir` ✓. Make it private (no `pub`) — keep the surface opaque. Hmm — but the user's list is "fresh, latest, existing methods" — no sessions_dir. Private ✓.

2. Change `Session::load`'s signature: `pub async fn load(session: &File) -> Result<Self, reason::Error>`; internally use `session.path()`:

```rust
    pub async fn load(session: &File) -> Result<Self, reason::Error> {
        use std::io::BufRead;

        let mut session = Self { ... };

        let path = session.path().to_path_buf();

        tokio::task::spawn_blocking(move || { ... use path ... })
```

Hmm — shadowing: the parameter is named `session`, and inside I construct a `Self` also named `session`. The current code has `path` as the parameter and builds `session`. If the parameter is `session: &File`, the local `let mut session = Self {...}` shadows it — that's confusing but legal… I'd rather rename the parameter or the local. Options:
- `pub async fn load(file: &File) -> ...` and internally `let path = file.path().to_path_buf();`. Clean: parameter `file`, local `session` stays as is.
- But the user said "make `Session`'s methods take `&File`" — the parameter name is free. `file: &File` is nice and readable.

Similarly `append(file: &File, events: ...)`:
- `let _lock = file::lock(file.path()).await;` — hmm, `file::lock(&path)` currently takes `impl AsRef<Path>` — `file.path()` is `&Path` ✓.
- `tokio::fs::try_exists(file.path())` ✓
- `tokio::fs::write(file.path(), created)` ✓
- `tokio::fs::OpenOptions...open(file.path())` ✓
- And move `create_dir_all` here (since main can no longer do `session.parent()`):
  ```rust
  // the session's directory may not exist yet
  if let Some(directory) = file.path().parent() {
      tokio::fs::create_dir_all(directory).await?;
  }
  ```
  Where? Before the try_exists (the directory must exist for the write). Currently main does it asynchronously with tokio::fs — should it stay async (tokio::fs) inside append, or move to a spawn_blocking section? append is mostly async tokio I/O (not yet in spawn_blocking — only the JSON serialization is). The create_dir_all there is a tokio::fs call (dispatches blocking internally) — one call per save, fine. Keep it as a tokio::fs call at the top of append. Hmm — or inside the serialize spawn_blocking closure? That closure only serializes. Keep the tokio::fs create_dir_all as the first statement (after the lock):

  ```rust
  let _lock = file::lock(file.path()).await;

  // The sessions directory is created on first use...
  if let Some(directory) = file.path().parent() {
      tokio::fs::create_dir_all(directory).await?;
  }
  ```
  Wait — `file::lock` + `file.path()` — the `file` parameter shadows the `file` module!! `use crate::file;` at the top + parameter named `file` → inside append, `file::lock` fails (the parameter shadows the module). The original code used `path: impl AsRef<Path>` and `file::lock(&path)` — no clash. If the parameter is `file: &File`, then `file::lock(file.path())` — `file::lock` — name resolution: `file` resolves to the parameter (type `&File`), and `file::lock` as a path → error (the parameter is not a module/path).

  So: either rename the parameter, or use `crate::file::lock`. Cleanest: name the parameter `session`? Then `file::lock(session.path())` — no shadowing of the `file` module. And the local in load `let mut session = Self{}` shadows the parameter `session` — load doesn't use `file::` so no issue, but the shadowing is icky:
  ```rust
  pub async fn load(session: &File) -> Result<Self, reason::Error> {
      let mut session = Self { ... };   // shadows the parameter
      let path = ??? -- the parameter is shadowed; I need the path first:
  ```
  Order: extract the path before shadowing:
  ```rust
  pub async fn load(session: &File) -> Result<Self, reason::Error> {
      let path = session.path().to_path_buf();

      let mut session = Self { ... };
  ```
  Works (path extracted first, then shadow). A bit awkward but compact. Hmm, or keep the parameter as `file` and qualify the module: `crate::file::lock(file.path()).await`. Hmm. Which is less confusing?

  Actually — wait. In append, `file` (the parameter) and `file::lock` (the module): the clash is real. In load, the `file` parameter + the local `let file = std::fs::File::open(...)` inside the closure also clash (the closure already binds a local `file` for the std File!). Look at load's closure: `let file = match std::fs::File::open(&path)` — a local `file` (std's File) exists. So naming the parameter `file` is a triple source of confusion.

  Decision: name the parameter **`session`** (it IS a session file — reads naturally: `Session::load(session: &File)`, `Session::append(session: &File, ...)`). Inside load, extract `let path = session.path().to_path_buf();` before shadowing with `let mut session = Self {...}`. Hmm — or rename the local instead? No — the local `session` is used all over inside the closure (it's moved). Keep `session` as the parameter, extract path first.

  Hmm, wait — actually, cleaner in load: don't shadow at all — the default-constructed local could be named `default`… no, `session` is the natural name and it's used ~10 times in the closure. The shadow with path extracted first is fine and common.

  For append: `let _lock = file::lock(session.path()).await;` ✓ no clash (`file` module is intact).

  And `open(session.path())` etc.

  Hmm — one more: `tokio::fs::try_exists(session.path())` — fine.

  Actually wait, reconsidering: `session: &File` — but then `session.path()` — `path()` is a private method on File (same crate ✓).

3. lib.rs exports: `pub use session::{File, Session};`? Hmm — currently: `pub use session::Session;` and the `pub mod session;` (main.rs imports `crate::core::session` for `session::Event`). So main.rs can do `core::session::File` via the module path without changing lib.rs. But is it cleaner to re-export File at the root? The root currently only re-exports `Session` and `Output`. `file` and `path` modules are pub. For consistency with `Session` (re-exported), also add `File` to the root re-export: `pub use session::{File, Session};`. Then main.rs: `use crate::core::{File, Session};`? Currently `use crate::core::Session;` — change to `use crate::core::{File, Session};` and drop the `session` import? No — main still uses `session::Event` (save() maps to `session::Event::ItemAdded`). Keep `use crate::core::session;` and add File to the Session import line: `use crate::core::{Session, ...}` hmm the import list: line 12 `use crate::core::Session;` → `use crate::core::{File, Session};`… wait, the ordering within braces: `{File, Session}` alphabetical ✓.

4. Test updates:
   - `a_session_loads_what_it_appends`: `let path = temp...PathBuf` → now need a `File`. But File's constructor `File(PathBuf)` — private field — the tests module is a child of the session module → can construct `File(path)` ✓ (same crate, parent's private fields visible from a child module? Privacy: `File.0` is private to the module where `File` is defined (the session module, i.e., the root of session.rs). `mod tests` is a child of that module → child modules can access ancestors' private items ✓. So `File(path)` works in tests.)

     Test:
     ```rust
     let file = File(std::env::temp_dir().join(format!("pick-session-test-{}", std::process::id())));
     std::fs::remove_file(file.path()).ok();
     let at = Session::append(&file, [...]).await.expect(...);
     let session = Session::load(&file).await.expect(...);
     std::fs::remove_file(file.path()).ok();
     ```
     `file.path()` — a private method callable from the tests module ✓.
     
   - `a_missing_file_loads_an_empty_session`: same treatment.
   - `concurrent_appends...`: `let file = File(path); ... Session::append(&file, ...)` — inside the loop it previously did `let path = path.clone()` per task → now `let file = file.clone()` per task.
     And `std::fs::read_to_string(&path)` → `std::fs::read_to_string(file.path())`.
   - `a_load_answers_the_calls_a_crash_left_open`: same treatment.
   - `a_fresh_path_stays_unique_and_orderly` → rename to `a_fresh_file_stays_unique_and_orderly`? Or keep the name — the name describes the property. Since the vocabulary changed, rename to match the new method names:
     - `a_fresh_file_stays_unique_and_orderly` — uses `File::fresh(&project)`; `std::fs::write(first.path(), "")`; stem assertion on `first.0.file_stem()`… hmm — `first.0` or `first.path().file_stem()`. Use `first.path().file_stem()`.
     - `the_latest_file_is_the_greatest_name` — `File::latest(&project)` == `Some(File(dir.join("20250914-103015123.jsonl")))`.
     - `a_missing_sessions_dir_has_no_latest_file`.
   - New test for `existing`:
     ```rust
     #[test]
     fn an_existing_session_resolves_by_name_and_path() {
         let project = std::env::temp_dir().join(format!("pick-sessions-d-{}", std::process::id()));
         let dir = File::sessions_dir(&project);
         std::fs::create_dir_all(&dir).unwrap();

         std::fs::write(dir.join("20250914-090000.jsonl"), "").unwrap();

         // By bare name in the sessions directory, with or without
         // the extension, and by path.
         assert_eq!(
             File::existing(&project, "20250914-090000"),
             Some(File(dir.join("20250914-090000.jsonl")))
         );
         assert_eq!(
             File::existing(&project, "20250914-090000.jsonl"),
             Some(File(dir.join("20250914-090000.jsonl")))
         );
         assert_eq!(
             File::existing(&project, ".pick/sessions/20250914-090000.jsonl"),
             Some(File(dir.join("20250914-090000.jsonl")))
         );

         // A name no session holds resolves to nothing.
         assert_eq!(File::existing(&project, "19700101-000000"), None);

         std::fs::remove_dir_all(&project).ok();
     }
     ```
     Wait — danger: `File::existing(project, "20250914-090000")` — the candidate `PathBuf::from("20250914-090000")` is relative to the CWD at test time = the workspace root (cargo runs tests with CWD = crate directory? For `cargo test -p pick-core`, CWD = core/). Does `core/20250914-090000` exist? No ✓. `project.join(name)` = fixture ✓ doesn't exist (no file without .jsonl) → the `dir.join(name)` candidate: fixture sessions dir + bare name → no (the file has .jsonl) → `dir.join(name + ".jsonl")` ✓ exists → returns it ✓.
     
     For `".pick/sessions/20250914-090000.jsonl"`: candidate 1 `PathBuf::from(...)` relative to CWD (core/).pick/sessions/... — doesn't exist (core has no .pick… wait — is there a `.pick` in core/? No. The repo root has `.pick/` (session.jsonl) — CWD for pick-core's tests = `core/` — `core/.pick/sessions/...` doesn't exist ✓). Candidate 2 `project.join(".pick/sessions/...")` = the fixture ✓ exists → returns it ✓.
     
     For `19700101-000000`: no candidate exists → None ✓. (Note: `19700101-000000.jsonl` in the fixture's dir? Not created ✓.)

   - `the_latest_file_is_the_greatest_name` — hmm, the name: "the latest file is the greatest name". OK.

   Also, the `File` constructor in the expected value of `latest`: `Some(File(dir.join("...")))` ✓.

5. **main.rs changes**:
   - Imports: `use crate::core::Session;` → `use crate::core::{File, Session};`
   - `resolve_session`:
     ```rust
     /// Resolves the argument of `--resume` to an existing session,
     /// exiting with a message when none exists.
     fn resolve_session(project: &Path, path: &str) -> File {
         File::existing(project, path).unwrap_or_else(|| {
             eprintln!("no such session: {path}");
             std::process::exit(1);
         })
     }
     ```
   - main's match:
     ```rust
     let session = match resume_path {
         Some(path) => resolve_session(&project, &path),
         None if resume => File::latest(&project).unwrap_or_else(|| File::fresh(&project)),
         None => File::fresh(&project),
     };
     ```
     (The closure `|| File::fresh(&project)` borrows `project` — fine, it's used immediately.)
   - `Pick` struct: `session: File,`
   - `Pick::new(prompt: Option<&str>, session: &File)`:
     ```rust
     let project = env::current_dir().unwrap_or_default();

     let mut pick = Self {
         ...
         session: session.clone(),
         ...
     };

     let load = Task::perform(
         async move {
             let session = session.clone();
             Session::load(&session).await
         },
         Message::SessionLoaded,
     );
     ```
     Hmm — the async move closure captures `session: &File` (the parameter reference)?? `async move { let session = session.clone(); ... }` — the async block captures the parameter `session: &File` (a copy of the reference, 'static? No — the reference's lifetime is that of the parameter — not 'static!). The `&File` parameter lives on the caller's stack; a future containing that reference can't be 'static → error.

     So we must first clone into an owned `File`, then the async block captures the owned clone:
     ```rust
     let session = session.clone();
     let load = Task::perform(
         async move { Session::load(&session).await },
         Message::SessionLoaded,
     );
     ```
     `async move` captures `session: File` by value ✓ 'static ✓; the inner `&session` borrows the captured owned value (owned by the future) ✓.
     
     And the struct field uses `session: session.clone()` (two clones — or reorder: build the struct with the clone, keep the local for load… the local `session` (owned) is moved into the async block → the struct must clone before: `session: session.clone()` in the literal, then `async move { Session::load(&session)... }` moves the original. ✓ one extra clone, fine.

   - `save()`:
     ```rust
     let session = self.session.clone();
     let new_last_saved = ...;

     Task::perform(
         async move {
             Session::append(&session, new_events).await?;
             Ok(new_last_saved)
         },
         Message::SessionSaved,
     )
     ```
     (create_dir_all moved to append ✓; the `?` on append's Result<(), reason::Error> → Result<usize, reason::Error> ✓.)

   - Does main.rs still use `PathBuf`? `project: PathBuf` field ✓, `home: Option<PathBuf>` ✓, resolve_session's parameter `&Path` ✓ → keep `use std::path::{Path, PathBuf};`.

6. **`File` doc** — "opaque": the struct's field is private; no public path accessor. Derives give Debug/Clone/Ord/PartialEq.

Edge case check — `Session::load`'s doc: "loads the session at `path`" → "loads the session at `file`…":
```rust
    /// Loads the session at `file`, or an empty session when the
    /// file does not exist yet: the first `append` creates it.
    pub async fn load(session: &File) -> Result<Self, reason::Error> {
```
Hmm "at `file`" vs parameter `session`: "loads the session at the given file"… let me phrase it:
```rust
    /// Loads the session in `file`, or an empty session when the
    /// file does not exist yet: the first `append` creates it.
```

Append's doc: append has no doc comment currently (the `file::lock` has an inline comment). Add a small doc? There's `// Appends to the same session are serialized, so the created frame is written by exactly one of them.` Keep as is, adjust `&path` → `session.path()`.

Wait — one more: `file::lock(&path)` → `file::lock(session.path())` — `lock(path: impl AsRef<Path>)` — `&Path` ✓.

And the `use std::path::{Path, PathBuf}` in session.rs — still needed: `Path` for the parameter of `fresh/latest/existing` and for `path() -> &Path` ✓; `PathBuf` for `sessions_dir`, the `File` field, candidates ✓.

`SystemTime` import ✓ no change.

7. Re-verify `Pick::new`'s load future — `Task::perform(f: impl Future + Send + 'static, to: impl Fn(Output))`. The async move block → 'static ✓ Send? `File` is Send (PathBuf) ✓, `Session::load`'s future is Send ✓.

8. One subtlety — `File`'s derive `Ord`: derived lexicographic order over PathBuf components — matches the previous tests ✓.

9. Should `File` implement `Display`? I decided no (tests use `.path()`).

Now — naming: `File` inside the `session` module — in main.rs it's `File` (imported from the core root). No clash with std::fs::File in main (main doesn't import std::fs::File) ✓. In session.rs: `use crate::file;` (module) + `pub struct File` — is there a clash?? The module `file` (lowercase) vs the type `File` (uppercase) — different names in Rust's module/type namespaces? Actually `use crate::file;` imports the module into the type namespace… modules live in the type namespace. `file` (lowercase) vs `File` (uppercase) — different identifiers, no clash ✓. In session.rs, `std::fs::File::open` — fully qualified ✓ no clash with our `File` (which would be `File` or `self::File`). Inside a load closure: `let file = std::fs::File::open(&path)` — a local `file` variable + the `crate::file` module used only in append (not in load) — load's closure doesn't use `file::` ✓.

Wait — inside `append`, the parameter `session: &File` — and `file::lock(...)` — the `file` module — the parameter name `session` doesn't shadow it ✓.

10. lib.rs: `pub use session::{File, Session};` — hmm, currently it's `pub use session::Session;`. Alphabetical: `{File, Session}` ✓.

Also verify: main.rs line 14 `use crate::core::session;` — for `session::Event`. Keep.

OK — is there anywhere else that calls `Session::load`/`append` with a PathBuf? Tests (updated), main (updated). Grep to be safe.

OK, let me write out all the edits. The File type + the signature changes in session.rs is a big restructure. Let me do it in careful edits:

**Edit A** (session.rs): Insert the `File` type before `pub struct Session`:
```rust
/// A session's file: an opaque handle to one of the project's
/// `.pick/sessions/*.jsonl` files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct File(PathBuf);

impl File {
    /// The directory in which the project's sessions live.
    fn sessions_dir(project: &Path) -> PathBuf {
        project.join(".pick/sessions")
    }

    ... fresh, latest, existing, path()
}

#[derive(Debug, Clone)]
pub struct Session {
```

**Edit B**: Remove the old `sessions_dir`/`fresh_path`/`latest_path` from `impl Session` (the block I added at the end of impl Session).

**Edit C**: `load`'s signature + the first lines.

**Edit D**: `append`'s signature + the lock + create_dir_all + try_exists + open path.

**Edit E**: lib.rs exports.

**Edit F**: Test updates (5 tests + 1 new).

**Edit G**: main.rs (imports, resolve_session, match, struct field, new, save).

OK let me write. For edit B, the old block:

```rust
    /// The directory in which the project's sessions live.
    pub fn sessions_dir(project: &Path) -> PathBuf {
        project.join(".pick/sessions")
    }

    /// The path of a new session, named by its start:
    ...
    pub fn latest_path(project: &Path) -> Option<PathBuf> {
        ...
    }
}
```
→ just `}` (end of impl Session).

The new File methods (moved from the impl Session block): fresh (Self), latest (Option<Self> + `.map(Self)`), existing (new), sessions_dir (private), path() (private).

Also update the fresh doc comment: "so `latest_path` can still tell them apart" → "so `latest` can still tell them apart".

Append changes in detail:

```rust
    pub async fn append(
        session: &File,
        events: impl IntoIterator<Item = Event> + Send + 'static,
    ) -> Result<SystemTime, reason::Error> {
        use tokio::io::AsyncWriteExt;

        // Appends to the same session are serialized, so the
        // `created` frame is written by exactly one of them.
        let _lock = file::lock(session.path()).await;

        // The sessions directory is created on first use.
        if let Some(directory) = session.path().parent() {
            tokio::fs::create_dir_all(directory).await?;
        }

        let now = SystemTime::now();

        if !tokio::fs::try_exists(session.path()).await? {
            ...
            tokio::fs::write(session.path(), created).await?;
        }

        ...
        let file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(session.path())
            .await?;
```

`session.path().parent()` — `Path::parent` ✓.

Load changes:
```rust
    /// Loads the session in `session`, or an empty session when the
    /// file does not exist yet: the first `append` creates it.
    pub async fn load(session: &File) -> Result<Self, reason::Error> {
        use std::io::BufRead;

        let path = session.path().to_path_buf();

        let mut session = Self {
            version: Version::current(),
            started_at: SystemTime::now(),
            items: Vec::new(),
        };
        ...
```
(The `let path = path.as_ref().to_path_buf();` line is removed.)

Doc wording: "loads the session in `session`" — awkward. Use:
```rust
    /// Loads a session from its file, or an empty session when the
    /// file does not exist yet: the first `append` creates it.
```

main.rs `new`:
```rust
    fn new(prompt: Option<&str>, session: &File) -> (Self, Task<Message>) {
        let project = env::current_dir().unwrap_or_default();
        let session = session.clone();

        let mut pick = Self {
            ...
            session: session.clone(),
            ...
        };

        let load = Task::perform(
            async move { Session::load(&session).await },
            Message::SessionLoaded,
        );
```

main's match:
```rust
    // Fresh by default; `--resume` picks up the newest session,
    // or the one at the path that follows the flag.
    let session = match resume_path {
        Some(path) => resolve_session(&project, &path),
        None if resume => File::latest(&project).unwrap_or_else(|| File::fresh(&project)),
        None => File::fresh(&project),
    };
```

resolve_session:
```rust
/// Resolves the argument of `--resume` to an existing session,
/// exiting with a message when none exists.
fn resolve_session(project: &Path, path: &str) -> File {
    File::existing(project, path).unwrap_or_else(|| {
        eprintln!("no such session: {path}");
        std::process::exit(1);
    })
}
```

save():
```rust
        let session = self.session.clone();
        let new_last_saved = self.last_saved + new_events.len();

        Task::perform(
            async move {
                Session::append(&session, new_events).await?;

                Ok(new_last_saved)
            },
            Message::SessionSaved,
        )
```

Import line 12: `use crate::core::Session;` → `use crate::core::{File, Session};`

OK let me write all the edits. Batch the independent ones. Order within session.rs matters for edit uniqueness, but each old_string is unique. Let's go.
