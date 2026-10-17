# publish-new-version

1. Set the new version in `Cargo.toml`, for example `version = "0.4.2"`.
2. Leave the notes under `## [Unreleased]` in `CHANGELOG.md` as they are. The release workflow reads them from there, so they must stay until the release is published.
3. Commit the changes locally, without pushing: `git commit -am "chore(release): 0.4.2"`.
4. Create a tag on that commit, without a `v` prefix: `git tag 0.4.2`.
5. Push only the tag: `git push origin 0.4.2`. The release workflow builds from this tagged commit.
6. Wait until the release workflow finishes. If it fails, delete the tag locally and on origin (`git tag -d 0.4.2` and `git push origin :refs/tags/0.4.2`).
   - If the failure is transient and nothing needs changing, go straight to step 4, so the tag is recreated on the existing commit. The release keeps its existing assets, so this is safe.
   - If the failure needs a fix that changes the built binaries, do not reuse the tag. The workflow keeps existing assets on a rerun, so the old binary would stay on the release. Use a new version number instead, starting again at step 1.
   - If the failure is a workflow-only fix that does not change the built binaries, keep the existing assets, and rerun the workflow on the same tag. Do not delete assets from a published release.
7. When the workflow is healthy, push the commit: `git push origin HEAD`.
8. Move the notes out of `[Unreleased]` in `CHANGELOG.md` into a new `## [0.4.2] - YYYY-MM-DD` section, leaving `[Unreleased]` empty. Commit and push this to the branch. Do this only after the release is published, so the workflow could still read the notes during the run.
9. Merge the pull request to `main`, so `main` matches the released code.
