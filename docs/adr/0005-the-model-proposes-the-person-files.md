# The Model proposes, the person files

A Model reads an Unfiled Note and makes a Proposal: title, tags, and one of the person's existing Folders or a new one. Nothing is filed until the person confirms or corrects it. The requirements want everything that comes in to be structured without typing it, and the deck files fully automatically; but the Model is a third party the app does not control (ADR 0003), and a second brain that silently misfiles loses trust.

The Folders are the person's own. The app brings no fixed scheme; only the Inbox is fixed. A folder someone already keeps in another tool can be opened as it is.

## Considered Options

- The Model files on its own right after Capture (rejected: unseen filing by a model nobody controls).
- Filing by hand only, the Model only answers (rejected: the structuring is what was asked for; this remains the behaviour with no Model connected).
- A fixed folder scheme, or no folders and tags alone (rejected: either fights a structure the person already has).
