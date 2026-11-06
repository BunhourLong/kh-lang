# T7 — review of the fix round (read-only)

Review ONLY the uncommitted changes: `git diff HEAD` and `git status`. That covers the new
untracked `npm/**` and `.github/workflows/release.yml`, and the staged deletion of
`crates/kh-lang-core/Cargo.lock`. Spec: `.kh-lang/prompts/T7-fix.md` (the brief) and
`.kh-lang/tasks.md#T7`. Do NOT edit files or commit.

Check, in priority order:
1. **Will a `v0.1.0` tag actually publish working installs on all 5 targets?**
   - Are the runner labels valid and not retired as of Nov 2026? Specifically check `macos-13`
     (GitHub retired the Intel macOS-13 image; consider `macos-15-intel`, or cross-building
     x86_64 on arm macOS) and `ubuntu-24.04-arm`.
   - Does the `.node` filename produced by each build match each `npm/<p>/package.json` `main`
     and what `index.js` requires?
   - Does `napi artifacts` find the downloaded files at the paths the workflow uses?
   - Does `prepublishOnly` (`napi prepublish -t npm`) publish the sub-packages before the root?
   - Is `NODE_AUTH_TOKEN` wired correctly? Do provenance and permissions work?
2. **Platform package metadata.** Check `os`/`cpu` on each package, and `libc: ["glibc"]` on
   the linux gnu packages (otherwise musl hosts install a glibc binary). Check versions are
   pinned to 0.1.0 and match the root `optionalDependencies`. Check each package has
   `license`, `repository` and `engines`.
3. **CI pack check.** Does it genuinely fail on a root `.node` file and on a missing
   optionalDependency? Is the gold guard still intact?
4. **Accuracy and scope.** Is the README install paragraph accurate? Are there changes
   outside the allowed files?

Output: numbered findings with `file:line`, each labelled BLOCKING or NIT, most severe first,
then a final line exactly `APPROVE` or `REQUEST-CHANGES`.
