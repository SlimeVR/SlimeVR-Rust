# Autoupdater

The SlimeVR autoupdater manages updating all the software for SlimeVR on a user's
computer. It does not handle updates of firmware.

It reads a `version.yaml` file from a github release to determine the versions of the
software to download, then fetches them and installs them.

Long term, we may use this to replace most or all of the business logic of the current
[SlimeVR Web Installer](https://github.com/SlimeVR/SlimeVR-Installer).

## Project Status
This is abandoned due to lack of interested developers.

When being actively developed, the Yaml description and serialization was complete
already.
