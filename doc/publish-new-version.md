# publish-new-version

1. Rename the old version everywhere to the new one, except for the changelog.
2. Have `Unreleased` still in the changelog.
3. Create a commit `ci: publish x.y.z` from that, but dont push it.
4. Push a version tag in the format `x.y.z`.
5. If everything looks good, remove the draft marker of the release.
6. Merge the PRs in [homebrew-vidmerger](https://github.com/tgotwig/homebrew-vidmerger) and [homebrew-linux-vidmerger](https://github.com/tgotwig/homebrew-linux-vidmerger).
7. Merge the commit into the `main` branch.
