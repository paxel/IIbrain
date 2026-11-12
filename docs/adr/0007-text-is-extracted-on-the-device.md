# Text is extracted on the Device, by the app itself, with no Google library

Extracted Text is made by the app on the Device where the Capture happened: photos and scanned pages with the text recognition Old Brain Fix uses on phone and desktop (RustO), recordings with Whisper, PDFs from their text layer and scanned pages through the same text recognition, linked pages by fetching and cleaning them on the Device. One engine per job gives the same text on phone and desktop, and full-text search (ADR 0006) then covers photos, recordings and PDFs with no Model connected. The concept paper names Google ML Kit and the deck a server pipeline; the first is a Google library in an app that ships outside Play, the second needs a server.

PDFs, images, plain text and Markdown files become searchable. Any other file is kept as an Attachment and found by its name and its Note only; office formats would each need a parser, and anything printable gets in through Print Capture.

For a link the app fetches the page itself and stops there: no archive services for blocked pages and no browser session inside the app. Print Capture from the person's own browser covers blocked and logged-in pages, and no address of theirs goes to an archive service.

## Considered Options

- Google ML Kit on Android (rejected: Google library; a second engine would be needed on the desktop).
- Handing photos to the connected Model (rejected: no text without a Model that can see, and Private photos would need a Local Model).

## Consequences

- The Whisper model is a download; the engines add native code to both apps.
- Still to be checked: which Android versions Print Capture and Whisper need. The app otherwise runs on whatever Flutter supports by default.
