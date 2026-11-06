# T7 — final review after fix round 2 (read-only)

Review the full uncommitted T7 change set: `git diff HEAD` + untracked `npm/**` and
`.github/workflows/release.yml`. Briefs: `.kh-lang/prompts/T7-fix.md`, `.kh-lang/prompts/T7-fix2.md`.
Prior findings: `.kh-lang/prompts/T7-review2.md` lists what was checked. Do NOT edit or commit.

1. Confirm that each round-2 item is resolved:
   - root `optionalDependencies` removed, and `npm ci` works with the unchanged lockfile;
   - `macos-15-intel` runner;
   - npm/* metadata (repository, publishConfig, os/cpu/libc, version 0.1.0);
   - `--no-gh-release`;
   - tag==version guard;
   - non-vacuous `\u`-escaped smoke test.
2. Check whether the round-2 edits introduced anything new. Look at YAML/job wiring
   (`needs`, matrix keys, artifact names), shell quoting on Windows (pwsh is the default
   shell there: is `shell: bash` set where bash syntax is used?), and the CI pack job still
   being correct.
3. Find any remaining reason a `v0.1.0` tag push with a valid `NPM_TOKEN` would fail to
   publish all six packages, or would publish a broken install.

Do not re-raise items already verified fine in round 2 unless the edits changed them.
Output: numbered findings with `file:line`, BLOCKING or NIT, then exactly `APPROVE` or
`REQUEST-CHANGES`.
