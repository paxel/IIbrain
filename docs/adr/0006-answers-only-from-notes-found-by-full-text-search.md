# Answers come only from Notes found by full-text search

For a Question the app searches the Brain by full text, hands the Notes it found to the Model as Sources, tells the Model to answer from them alone, and lists the Sources with every Answer. When the search finds nothing, the app says so and asks no Model. Full-text search works offline on every Device with nothing connected; search by meaning needs an embedding model, which would be one more third party and, for Private Notes, a local one.

A Saved Answer is a Note marked as written by a Model, linking its Sources, and can serve as a Source again.

## Considered Options

- Vector search (sqlite-vec), as in the deck (rejected for the first version; it can be added later without touching the files, since the index is rebuilt from the folder, ADR 0001).
- Letting the Model add what it knows itself, marked as such (rejected: the strictly grounded Answer is the concept's central promise).
- Never using Saved Answers as Sources (rejected: condensing knowledge for later is the point; the mark and the links make a wrong one traceable).

## Consequences

- The app can instruct a third-party Model but cannot make it obey; the listed Sources are how the person checks an Answer.
- A Question in other words than the Notes use finds less than a search by meaning would.

## How a Question becomes a search (#6)

The app derives the search itself and asks no Model for it: the Question is split into words, the filler words of English and German are dropped, and the Brain is searched for Notes holding any of the remaining words, best match first; the first eight become the Sources. So the rule stands as written: when the search finds nothing, no Model is asked, and a Question never leaves the Device on its own.

Considered: letting the Model write the search terms (rejected: every Question would go to the Model before the app knows whether anything fits, and with a Cloud Model the Question itself can be private).
