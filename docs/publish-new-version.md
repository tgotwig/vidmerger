# publish-new-version

1. Bump `version` in `Cargo.toml`, commit it, and tag it: `git tag 0.4.2` (no `v`).
2. Push the tag: `git push origin 0.4.2`. This creates a draft release and builds the assets.
3. If the run fails, delete the tag (`git push origin :refs/tags/0.4.2`) and retry from step 1.
4. Check the draft, then publish it: `gh release edit 0.4.2 --draft=false`.
5. Bump the Homebrew formulas by hand (`version`, `url`, `sha256`) in both tap repos.
6. Move the `[Unreleased]` notes in `CHANGELOG.md` into a `## [0.4.2]` section.
