# Shipped outside the stores, through our own tap, bucket and F-Droid repo

IIbrain is not published on Google Play or the App Store. It ships through catalogues we run ourselves, as our other apps do: an own Homebrew tap (Linux, macOS), an own Scoop bucket (Windows) and the own F-Droid repo (Android). An app that handles private notes and talks to models would mean store reviews and store rules we do not want to work under. This constraint stands above the requirements papers: where they need something only a store or Google's services provide, they bend.

The source is public under "Apache-2.0 OR MIT". The Linux desktop links LGPL code statically, as Old Brain Fix's does, so every released build must have public source anyway.

## Consequences

- There is no iPhone app: without the App Store an iPhone app has no ordinary way to its user.
- Nothing may depend on Google Play services being present on the phone.
- The app is called "IIbrain" in the repo, the packages and under the icon.
