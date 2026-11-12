# The app ships no Model; the person connects one through a single interface

IIbrain contains no language model and no model runtime. The person connects Models through one interface, an OpenAI-compatible HTTP endpoint (address, key, model name), and declares each as a Local Model or a Cloud Model. That one form reaches a cloud provider, a small model the person runs, and a model at home alike, so the app offers the variety without owning any of it. The concept paper wanted Gemini Nano on the phone and the deck a server model with a small on-device fallback; both would have bound the app to particular hardware and to Google's libraries, against shipping outside Play.

## Considered Options

- An on-device model on the phone (rejected: runs on few phones, brings Google libraries or a large download).
- Routing between a home server model and an on-device model, as in the deck (rejected: the app would have to solve the connection home).
- Vendor-specific interfaces from the start (rejected: one compatible form covers the cases).
- No ask screen at all, only outside agents reading the folder (rejected: asking from the phone is the heart of the concept; outside agents can read the folder anyway, ADR 0001).

## Consequences

- With no Model connected the app still captures, files by hand and searches.
- A model that runs inside the phone without any server is not reachable through this interface.
- The addresses and keys of Models belong to a Device and never enter the synced folder.
