# publish-new-version

The release workflow (`release.yml`) creates a **draft** GitHub release on a tag push and attaches the binaries to it. It does not publish the release and does not update the Homebrew taps. Both are manual steps below.

## Release

1. Set the new version in `Cargo.toml`, for example `version = "0.4.2"`.
2. Leave the notes under `## [Unreleased]` in `CHANGELOG.md` as they are. The release workflow reads them from there to fill the draft.
3. Commit the changes locally, without pushing: `git commit -am "chore(release): 0.4.2"`.
4. Create a tag on that commit, without a `v` prefix: `git tag 0.4.2`.
5. Push only the tag: `git push origin 0.4.2`. The workflow builds from this tagged commit.
6. Wait until the workflow finishes. If it fails, delete the tag locally and on origin (`git tag -d 0.4.2` and `git push origin :refs/tags/0.4.2`), then:
   - If nothing needs changing, go straight to step 4 to recreate the tag on the existing commit.
   - If the fix changes the built binaries, do not reuse the tag. The workflow keeps existing assets on a rerun, so the old binary would stay on the release. Use a new version number and start again at step 1.
   - If the fix only changes the workflow and not the binaries, keep the existing assets, commit the fix, and recreate the tag on the new commit at step 4.
7. Check the draft release on GitHub: all assets are attached, and the notes look right.
8. Publish the draft by hand: `gh release edit 0.4.2 --draft=false`.
9. Update the Homebrew formulas by hand. In each tap repo (`homebrew-vidmerger` and `homebrew-linux-vidmerger`), change `version`, `url`, and `sha256` in `vidmerger.rb` to the new release, then commit and push to `master`.
10. Push the version commit: `git push origin HEAD`.
11. Move the notes out of `[Unreleased]` in `CHANGELOG.md` into a new `## [0.4.2] - YYYY-MM-DD` section, leaving `[Unreleased]` empty, and commit it. Do this after the release is published.
12. Merge the pull request to `main`, so `main` matches the released code.
