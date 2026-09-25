// Installs this repository's git hooks, which nothing but a Claude Code session would otherwise
// install. `.claude/settings.json` does the same through a `SessionStart` entry; this is the same
// arrangement for OpenCode, and it exists for the reason ADR-0005 records — a commit made with the
// hooks absent runs no gate and says nothing, which is worse than a hook that fails loudly.
//
// The body of a plugin runs at startup, which is the moment `SessionStart` fires in the other
// harness. Two choices in the command below are deliberate. `--local` and `-C` are both there so
// that a session whose working directory is not a checkout writes nothing at all: a bare
// `git config` outside a repository would set the global config and change every other repository
// on the machine. A failure here is logged and swallowed, because the boundary that actually holds
// is CI and this is fast feedback in front of it.

export const InstallGitHooks = async ({ client, $, worktree }) => {
  try {
    await $`git -C ${worktree} config --local core.hooksPath .claude/git-hooks`
  } catch (error) {
    await client.app.log({
      body: {
        service: "monospace",
        level: "warn",
        message:
          "could not set core.hooksPath; commits from this session will run no gate. Run: git config core.hooksPath .claude/git-hooks",
        extra: { worktree, error: String(error) },
      },
    })
  }
}
