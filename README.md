# Dive Lens

Dive Lens links your underwater photos in Adobe Lightroom Classic with your dive log.

- **Create collections**: finds the photos taken during each dive from their capture time, creates one collection per dive, and tags every photo with the dive data, the dive site, and the **depth, water temperature and elapsed time at the moment the photo was taken**.
- **Tag selection**: tags the photos selected in Lightroom with a dive's data and/or a dive site.

Dive Lens reads your dive log directly, read-only. Nothing to export.

## Status

Version 2 is in development. It replaces the 0.x plugin with a dedicated desktop app.

| | V1 of the new app |
|---|---|
| Dive logs | MacDive, Subsurface |
| Platforms | macOS (Intel and Apple Silicon), Windows |
| Lightroom | Lightroom Classic 14+ |

BlueDive and Submersion are planned next.

The product specification (in French) is in [`docs/spec-v1.md`](docs/spec-v1.md).

## License

Dive Lens is free and open source, under the [GNU General Public License v3.0](LICENSE).

It includes [libdivecomputer](https://www.libdivecomputer.org) (LGPL 2.1, in `third_party/libdivecomputer`) to decode dive-computer data.

## Building

Clone with `git clone --recursive`, or run `git submodule update --init` after cloning.

## Contributing

Contributions are welcome. [`CONTRIBUTING.md`](CONTRIBUTING.md) explains how the project is built, the commands, and the rules for branches, commits and pull requests.

To report a bug or ask for another dive application, use the [issue templates](https://github.com/Ventrilok/divelens/issues/new/choose). The repository is public: never attach a real dive log, a dive-log database or the Dive Lens log files. If the log files are needed, you will be asked for them: send them only by e-mail to [divelens@querton.com](mailto:divelens@querton.com).

To report a security vulnerability or an exposed secret, see [`SECURITY.md`](SECURITY.md): privately, never in a public issue.
