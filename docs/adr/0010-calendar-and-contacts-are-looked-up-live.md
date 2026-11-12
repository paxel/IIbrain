# Calendar and contacts are looked up live on the phone and never stored

When a Question needs the calendar or the contacts, the phone reads them at that moment (a Live Lookup) and hands what it found to the Model with the Sources. Nothing of them is written into the Brain, and the desktop knows nothing of them. The deck instead copies them into the folder as Markdown Notes; that would put other people's data on every Device and in front of whoever hosts the folder (ADR 0001).

Each of the two has its own switch for Cloud Models, "local only" at the start (ADR 0004).

## Consequences

- A Question about an appointment or a person can only be answered on the phone.

## Without tool calls (#7)

A Live Lookup never depends on the Model calling a tool, so it works with every endpoint. The ask screen of the phone has two switches, calendar and contacts, off at the start of every Question. With calendar on, the phone reads the events of the next fourteen days; with contacts on, it reads the contacts whose names share a word with the Question. What it read goes to the Model beside the Sources, marked as calendar or contacts, and through the same gate (ADR 0004).

Considered: offering the Model tools to query calendar and contacts (rejected: many OpenAI-compatible endpoints offer none, and the Model would decide what of other people's data it reads).
