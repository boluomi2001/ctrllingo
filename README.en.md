<p align="center">
  <img src="assets/brand/ctrllingo-mark.png" alt="CtrlLingo · 译点 mark" width="160">
</p>

<h1 align="center">CtrlLingo · 译点</h1>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
  <img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" alt="GPL-3.0-or-later">
  <img src="https://img.shields.io/badge/version-0.1.0-orange" alt="Version 0.1.0">
</p>

CtrlLingo · 译点 is an independent Windows fork based on the Qingjian pinyin input method. Version **0.1.0** is a prerelease for Windows x64, with an x86 TSF component for 32-bit applications. On 2026-10-23, the user reported that all features worked in their testing. The dependency security review has not passed, so this remains a prerelease. See the [acceptance record](docs/RELEASE-VALIDATION.md) for verification scope.

## Features

- Pinyin lookup, candidate selection, learning, and the local sentence model run on your device. Pinyin lookup works offline.
- Press Ctrl to request a translation of the **highlighted Chinese candidate in the current pinyin composition**. This does not read or translate text selected in another application. Languages: English, Japanese, German, Spanish, and Vietnamese.
- Long-sentence reordering is conservative and aims to preserve your sentence choice. Sentence learning uses your selections to adjust future candidates.
- Cloud prediction is off by default and does not send automatic requests. On-demand translation is a separate explicit request; when triggered, it sends the highlighted candidate to the compatible API you configured. You provide the endpoint and API key, and the provider may charge for requests.

## Start typing in three steps

1. After installation, press **Win + Space** and select “译点 CtrlLingo” from the Windows input-method list.
2. Type a sentence in pinyin and move the highlight to the Chinese candidate you want.
3. Release **Ctrl** on its own to request that candidate's translation. When it arrives, press **Space** to commit it. Press Ctrl to switch back to the Chinese candidate; after changing the composition or highlight, press Ctrl again to request the new candidate's translation.

If no compatible endpoint and API key are configured, no cloud translation will be returned. Enter the API key on the Settings app's Cloud Services page. Pressing Ctrl is required for this translation request; turning cloud prediction off does not disable it.

## Download, installation, and uninstall

Download `CtrlLingo-0.1.0-windows-x64-setup.exe` from [GitHub Releases](https://github.com/boluomi2001/ctrllingo/releases). Version 0.1.0 remains a prerelease; release conditions and known issues are listed in the [acceptance record](docs/RELEASE-VALIDATION.md). The installer is unsigned. Check the release's `SHA256SUMS.txt`; do not disable Windows security protections to bypass a prompt.

Run the installer and follow its prompts. It registers both x64 and x86 TSF components. If an application was already open, reopen it before selecting the input method, or sign out and back in.

Uninstall from the Start menu's “Uninstall CtrlLingo” shortcut or Windows Settings → Apps. Uninstall removes the program and TSF registrations, but retains your configuration, API key, learning data, statistics, and runtime logs. To remove personal data as well, delete `%APPDATA%\CtrlLingo` and `%LOCALAPPDATA%\CtrlLingo` after uninstalling.

The program is installed under `%ProgramFiles%\CtrlLingo`; its background server and settings app are `ctrllingo-server.exe` and `ctrllingo-settings.exe`. The settings shortcut and system tray identify the product as “译点 CtrlLingo”. Configuration, credentials, and learning data are stored under `%APPDATA%\CtrlLingo`; runtime logs are under `%LOCALAPPDATA%\CtrlLingo\logs`; the machine-wide TSF icon is under `%ProgramData%\CtrlLingo`.

## Cost estimates and account balance

The cost view estimates usage locally from recorded requests and a price table; it is not a provider invoice. Estimates may be incomplete when usage data is missing, prices change, cache billing differs, or a custom compatible service is used. The provider account balance may include activity and top-ups from other applications, so it is not CtrlLingo's spend. Balance lookup is available only for the supported official DeepSeek API.

## Privacy and updates

On-demand translation sends the currently highlighted Chinese candidate to your configured provider. Cloud prediction sends nearby composition text when enabled. These features share the API configuration but have different triggers. The API key is stored in a local configuration file. Input logging is off by default. See the [privacy notice](docs/PRIVACY.md) for details.

The Windows build does not check for updates from Qingjian upstream. Open this project's [GitHub Releases](https://github.com/boluomi2001/ctrllingo/releases) manually from the About page to look for updates.

## Build and test

Builds require Rust 1.96, the MSVC C++ toolchain, and Inno Setup 6. From the repository root, run in PowerShell 7 or Windows PowerShell 5.1:

```powershell
$env:CTRLLINGO_ISCC = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
.\apps\windows\installer\build-ctrllingo.ps1
```

The script fetches and verifies product data using [`tools/release/data.lock`](tools/release/data.lock). By default it writes the installer, source archive, and `SHA256SUMS.txt` to `target\ctrllingo-release`. The files are named `CtrlLingo-0.1.0-windows-x64-setup.exe` and `CtrlLingo-0.1.0-source.zip`. To run Windows-related crate tests:

```powershell
cargo test --locked -p qingjian-windows-server -p qingjian-windows-tsf -p qingjian-settings
cargo test --locked -p qingjian-core
```

These are build and verification instructions; they do not claim that release acceptance has passed.

## Origin and license

CtrlLingo is maintained by `boluomi2001` and is based on [Qingjian](https://github.com/qingjian-team/qingjian), commit [`c97ff2b61933d6fcd845ee4f4f470faf3ea83bf6`](https://github.com/qingjian-team/qingjian/commit/c97ff2b61933d6fcd845ee4f4f470faf3ea83bf6), with the fourth iteration of changes from this working tree. It is not an official Qingjian product and does not claim original authorship of the entire input engine. See [`docs/UPSTREAM.md`](docs/UPSTREAM.md) and the [upstream README copy](docs/upstream/README.zh-CN.md) for attribution.

Code is provided under [GPL-3.0-or-later](LICENSE). Bundled data and some components may have separate origins and licenses. The inherited macOS and Linux source has not been accepted as a CtrlLingo release for those platforms.

## Feedback and contributions

- [Report a bug or suggest a feature](https://github.com/boluomi2001/ctrllingo/issues/new/choose)
- [Security policy](SECURITY.md) (report security issues privately)
- [Support](SUPPORT.md) · [Contributing](CONTRIBUTING.md)

Dependency audit currently fails on three inherited ZIP extraction advisories. User-reported feature testing does not change that result; this prerelease does not claim a clean security audit. See [known issues](docs/KNOWN-ISSUES.md) and the [acceptance record](docs/RELEASE-VALIDATION.md) for evidence boundaries.
