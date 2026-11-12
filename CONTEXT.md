# IIbrain

A private second brain: everything a person wants to keep goes in from phone or computer, gets structured, and can be asked later, with answers drawn only from the person's own notes. Phone and desktop are equal; the notes are plain files the person owns, and the language model is one the person connects.

## Language

### Keeping

**Brain**:
Everything one person keeps in IIbrain: the Notes with their Attachments, in the folders the person made.
_Avoid_: vault, repo, knowledge base, archive

**Note**:
One kept thing: its text, what is known about it (title, tags, when and how it was captured) and its Attachments.
_Avoid_: entry, document, item, memo

**Attachment**:
A file kept with a Note: a photo, a PDF, a recording or any other document.
_Avoid_: binary, blob, asset

**Extracted Text**:
Text the app itself read out of an Attachment or a linked web page: from a photo or scanned page, from a PDF, from a recording, from a page behind a link. Only the app writes it; it makes the Attachment searchable.
_Avoid_: OCR (in the UI), caption, content

**Folder**:
A place in the Brain the person made to hold Notes. A Folder can be Private.
_Avoid_: category, notebook, area, topic

**Inbox**:
The one fixed place of a Brain, where Unfiled Notes wait.
_Avoid_: unsorted, staging, queue

### Capturing

**Capture**:
The act of putting something into the Brain: typed text, a photo, a recording, a link, a file, something shared in from another app, or a Print Capture. A Capture makes an Unfiled Note.
_Avoid_: import, ingest, add

**Print Capture**:
A Capture made by choosing IIbrain as the printer in another app on the phone; the printed pages arrive as a PDF Attachment.
_Avoid_: virtual printer, print-to-brain

**Unfiled**:
A Note that still waits in the Inbox, with no confirmed title, tags and Folder.
_Avoid_: unsorted, new, raw

**Proposal**:
A Model's suggestion of title, tags and Folder for an Unfiled Note. It changes nothing until the person confirms or corrects it.
_Avoid_: suggestion (in the code), auto-filing, classification

**Filing**:
Confirming where an Unfiled Note belongs, with or without a Proposal. A filed Note has left the Inbox.
_Avoid_: sorting, organising, structuring

### Asking

**Model**:
A language model the person connected to the app. IIbrain brings none of its own.
_Avoid_: AI, LLM, assistant, engine

**Local Model**:
A Model the person declared as running under their own control, at home or on their own machine. The app takes the person's word for it.
_Avoid_: on-device model, offline model

**Cloud Model**:
A Model the person declared as run by someone else.
_Avoid_: remote model, online model, API

**Question**:
What the person asks the Brain in their own words.
_Avoid_: query (that is the search behind it), prompt, chat message

**Source**:
A Note the app found for a Question and handed to the Model. Every Answer lists its Sources.
_Avoid_: context, reference, citation, hit

**Answer**:
What a Model wrote in reply to a Question, from the Sources alone.
_Avoid_: response, result, reply

**Saved Answer**:
An Answer the person kept as a Note. It is marked as written by a Model and links its Sources.
_Avoid_: summary, generated note, synthesis

**Live Lookup**:
Reading the phone's calendar or contacts at the moment a Question needs them. Nothing of them is kept in the Brain.
_Avoid_: PIM sync, calendar import, contact import

### Privacy

**Private**:
A mark on a Note or a Folder: what carries it reaches only a Local Model, never a Cloud Model. An Unfiled Note is Private or not as the person's setting says; a Saved Answer with a Private Source is Private.
_Avoid_: secret, sensitive, locked, hidden (a Private Note is neither hidden nor encrypted)

### Devices

**Device**:
One installation of IIbrain, on a phone or a computer. The Models a Device knows and their keys belong to that Device alone.
_Avoid_: client, node, instance

**Sync Tool**:
A program of the person's own choice that carries the Brain between their Devices. It is not part of IIbrain.
_Avoid_: sync (as something IIbrain does), backend, cloud

**Conflict**:
Two versions of one Note, left side by side by the Sync Tool after two Devices changed it. The person keeps one or merges them.
_Avoid_: duplicate, collision, clash
