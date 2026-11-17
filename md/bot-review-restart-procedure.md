# Restarting the CodeRabbit review on the fork's pull requests

Measured on 2026-11-17 while restarting the review on all three open fork PRs
(#11837 quick-selection mode buttons, #11971 move-tool cross-document transfer,
#12452 localized Japanese fonts).

## The commands

```sh
# Token first (gh is not logged in on this machine).
$cred = "protocol=https`nhost=github.com`n"
$env:GH_TOKEN = (($cred | git credential fill) | Select-String '^password=(.*)$').Matches[0].Groups[1].Value

# Incremental review: reviews only commits CodeRabbit has not reviewed yet.
gh pr comment <number> --repo storytold/photocraft --body "@coderabbitai review"

# Full review: re-reviews the whole PR, even commits that were already reviewed.
gh pr comment <number> --repo storytold/photocraft --body "@coderabbitai full review"
```

A push to the PR branch also triggers a review automatically (this is how #11971
restarted on 2026-11-17 after `3ede773642` landed on
`feat/move-tool-selection-drag-transfer`).

## The reply decides, not the timestamp

CodeRabbit always answers a command with a comment. **A fresh comment timestamp is
not proof that a review restarted** — read the body:

| Reply body | Meaning |
|---|---|
| `Action performed` / `Review triggered.` | started (incremental; only unreviewed commits) |
| `Action performed` / `Full review triggered.` | started (everything re-reviewed) |
| `Action not completed` / `No files to review.` | **not started** — every commit was already reviewed; the incremental command refused |

Measured on 2026-11-17: `@coderabbitai review` on #11837 and #12452 replied
"No files to review" (both PRs had no unreviewed commits), while the same command
on #11971 (which had a fresh push) replied "Review triggered." `@coderabbitai
full review` then restarted #11837 and #12452 ("Full review triggered." at
04:49Z).

## Verify a restart actually happened

```sh
# The LAST CodeRabbit comment must say "Review triggered." / "Full review triggered."
# ("No files to review" means it did not restart — use the full review command.)
gh pr view <number> --repo storytold/photocraft --json comments --jq `
  '[.comments[] | select(.author.login=="coderabbitai")] | last | .body'

# Or count unresolved review threads (0 after a clean review round).
gh api graphql -F query=@q.graphql `
  --jq '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)] | length'
```

So the working sequence for "restart the review on every open fork PR" is:
`@coderabbitai full review` on each PR, then check each reply says
"Full review triggered." — a timestamp or an acknowledgment comment proves
nothing.
