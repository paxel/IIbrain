# The app does not sync; the person's own Sync Tool moves the folder

IIbrain reads and writes a local folder and nothing else. What carries that folder between Devices is the person's choice and the person's business: Syncthing, a Google Drive or FTP sync app, a home machine. The requirements name Google Drive, a home server and decentral sync side by side; instead of picking one or building connectors for each, the app stays out of it and holds no protocol, login or server address for sync.

## Considered Options

- Built-in connectors (Google Drive API, WebDAV, FTP) (rejected: every protocol and login would live in the app).
- A local folder plus a built-in connector for Google Drive only, because Android has no Drive folder sync of its own (rejected for the same reason; the person installs a sync app).
- Old Brain Fix's device-to-device sync (rejected with its log format, ADR 0001).

## Consequences

- The phone holds the whole Brain. The concept paper's cache that drops originals from the phone and fetches them back is not possible: a file removed from a synced folder is removed everywhere.
- Two Devices changing one Note leave two files. The app recognises such pairs as a Conflict and shows both versions to keep one or merge; how a pair is named depends on the Sync Tool.
- Reaching a machine at home from outside is not the app's concern.
- Still to be checked before anything is built on it: how an Android app installed outside Play keeps read and write access to a folder a sync app also writes, and whether the core can use that folder by plain paths.
