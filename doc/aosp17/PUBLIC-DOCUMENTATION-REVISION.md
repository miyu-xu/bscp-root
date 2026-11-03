# Current AOSP 17 release documentation

AOSP 17 is the published default source baseline. See DEPLOYMENT.md for prerequisites, configuration, build boundaries and rollback. This bundle supersedes the initial preparation descriptions. It contains no private deployment configuration.

Run `python -B verify_release_assets.py --directory <download-directory> --manifest release-assets-public.json` after downloading the named original assets. Replace the directory argument; it is not a literal installation path. This only verifies bytes and never downloads, extracts or executes an asset. It does not remove personal paths from legacy attachments.

The included installed-deployment verifier is a reviewed copy of `releases/aosp17/verify_installed.py`; run that version from the current repository checkout because its sibling production controllers are required. It requires `--deployment-config` pointing to the original private binding. Missing configuration is rejected; this is not a generic installer.

The original 28 assets were individually downloaded and SHA256-verified. Thirteen original archives contain environment-identity matches, including binary build/debug paths. Existing Git history and unmodified source closures may retain them. No old attachment has been silently deleted or overwritten. Binary redaction requires source/build-path remapping, rebuild and affected acceptance; not byte editing. Original evidence remains private and unchanged.
