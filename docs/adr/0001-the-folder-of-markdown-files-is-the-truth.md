# The folder of Markdown files is the truth, the database only an index

A Note is one Markdown file with YAML frontmatter, its Attachments are files beside it, and the Brain is the folder that holds them. The app's database (full-text index included) is rebuilt from the folder and never holds anything the folder lacks. All three sources of the requirements ask for open, readable files that Obsidian or a Model of the person's choice can use without IIbrain, and a Brain must outlive the app.

## Considered Options

- A JSON manifest per Note, as the concept paper says (rejected: the same paper names Obsidian as a reader, and Obsidian reads Markdown, not manifests).
- A Markdown file with a JSON sidecar for the metadata (rejected: two files per Note that can drift apart).
- Files encrypted by the app (rejected: no other tool could read them; see the consequences).
- Old Brain Fix's signed, encrypted entry log, to share one format with it (rejected: it ties IIbrain to that log and its versioning; Notes reach Old Brain Fix through the system share sheet like any other app).

## Consequences

- Whoever hosts the folder can read every Note. The Private mark (ADR 0004) governs Models only. Privacy toward a file host is decided by where the person puts the folder.
- Everything the app needs to know about a Note must fit into its file; the app keeps no state of a Note anywhere else.
- Other tools may change the files at any time; the app has to take the folder as it finds it.
