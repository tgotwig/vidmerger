# publish-new-version

1. Rename the old version everywhere to the new one, except for the changelog.
2. Update [README.md](../README.md) with changes from the [CHANGELOG.md](../CHANGELOG.md).
3. Have `Unreleased` still in the changelog.
4. Create a commit `ci: publish x.y.z` from that, but dont push it.
5. Push a version tag in the format `x.y.z`.
6. If everything looks good, remove the draft marker of the release.
7. Merge the PRs in [homebrew-vidmerger](https://github.com/tgotwig/homebrew-vidmerger) and [homebrew-linux-vidmerger](https://github.com/tgotwig/homebrew-linux-vidmerger).
8. Merge the commit into the `main` branch.
