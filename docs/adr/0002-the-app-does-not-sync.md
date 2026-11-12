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

## Folder access on Android (#2)

The phone app asks for all-files access: the `MANAGE_EXTERNAL_STORAGE` permission, granted by the person on the system's settings page (`ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION`), checked with `Environment.isExternalStorageManager()`. From Android 11 on it gives read and write access to all of shared storage by direct file paths, which is where a Sync Tool keeps its folders, so the core uses the Brain's folder with the ordinary file calls it uses on the desktop. Up to Android 10 the storage permissions give the same paths (`READ_EXTERNAL_STORAGE` and `WRITE_EXTERNAL_STORAGE`, with `requestLegacyExternalStorage` on Android 10). Google Play limits all-files access to a few kinds of apps; shipping outside Play (ADR 0009) is what makes it available here. Sources: Android's "Manage all files on a storage device" and "Access documents and other files from shared storage".

- The system's folder picker (`ACTION_OPEN_DOCUMENT_TREE`) was rejected: it hands out content URIs, not paths, so the core would need a second way into the folder, it is slow over many files, and it cannot grant the root of the storage or `Download`.
- The app's own directories under `Android/data` stay out of reach of other apps on Android 11 and later, so a Brain cannot live there if a Sync Tool is to carry it.
- Not tried on a device yet: the finding rests on Android's documentation.
