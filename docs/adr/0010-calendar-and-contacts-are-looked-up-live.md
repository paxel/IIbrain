# Calendar and contacts are looked up live on the phone and never stored

When a Question needs the calendar or the contacts, the phone reads them at that moment (a Live Lookup) and hands what it found to the Model with the Sources. Nothing of them is written into the Brain, and the desktop knows nothing of them. The deck instead copies them into the folder as Markdown Notes; that would put other people's data on every Device and in front of whoever hosts the folder (ADR 0001).

Each of the two has its own switch for Cloud Models, "local only" at the start (ADR 0004).

## Consequences

- A Question about an appointment or a person can only be answered on the phone.
- Still to be checked: whether a Live Lookup needs the Model to call a tool, and what happens with an endpoint that offers none.
