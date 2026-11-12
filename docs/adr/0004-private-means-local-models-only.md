# Private means Local Models only, and the safe state comes first

The one limit the requirements state outright is that not everything may go to a cloud AI. A Note or a Folder can therefore be marked Private, and what is Private reaches only a Model the person declared local. Where no mark exists yet, the safe state is the default and the person may open it:

- An Unfiled Note is Private unless the person's setting says otherwise, so a fresh Capture does not reach a Cloud Model for its Proposal before anyone could mark it.
- A Saved Answer with a Private Source is Private; the person may lift the mark by hand. Otherwise a Private Note would leave one step later through its summary.
- Live Lookups of calendar and contacts each have a switch, set to "local only" at the start.

## Considered Options

- No gate in the app, privacy by connecting only a Local Model (rejected: "not everything" means some things yes and some no).
- Showing before every send what would leave and asking (rejected: every Question becomes a dialog).

## Consequences

- The app takes the person's word that a Model is local; it cannot check where an address leads.
- The mark governs Models only. It hides nothing on the Device and nothing from whoever hosts the folder (ADR 0001).
- A person with only a Cloud Model gets no Proposals for Unfiled Notes until they change the setting or file by hand.

## Where the marks are kept (#5)

All of them lie in the Brain's folder, so that a mark set on one Device holds on every other:

- A Note's mark is `private: true` in its frontmatter.
- A Folder's mark is an empty file `.iibrain-private` in that Folder; it moves with the Folder when another tool renames it.
- The setting for Unfiled Notes is `unfiled_private` in `.iibrain/brain.yaml` at the root of the Brain; a missing file or key means Private.
- A Capture made while Unfiled Notes are open is written without a mark; one the person marked carries `private: true` from the start. So an Unfiled Note is Private when the setting says so or its own mark does.

Considered: keeping the marks as settings of a Device (rejected: a Capture that is Private on the phone could reach a Cloud Model from the desktop's Inbox). The cost is that whoever can write into the folder can lift a mark; that party can read every Note anyway (ADR 0001).
