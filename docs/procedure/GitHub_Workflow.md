# Team Development Standards

## 1. Before Every Workday

- **Sync with `main`:** Fetch the latest changes from `origin/main` and merge them into your local branch before starting work to account for changes merged by teammates.
- **Check for PRs to review:** Check the `PR Review Pings` thread in the Software channel on Discord for new pull requests that need review. Reviewing PRs and unblocking teammates is an important part of keeping development moving.

## 2. Writing Issues

- **Follow issue guidelines:** Refer to [Writing GitHub Issues](/docs/procedure/Writing_GitHub_Issues.md) for guidelines on writing clear, well-scoped issues with appropriate subsystem labels.

## 3. Picking Up an Issue

- **Two-month limit:** An issue may be held by one person for a maximum of two months. If you cannot complete it within that timeframe, communicate with the team to reassess ownership or priorities.

## 4. Working on an Issue

- **Follow branch naming conventions:** Name branches using the format `issueNumber-FirstnameLastInitial/short-description-of-the-issue`.

  Example: `58-CelinaH/address-TODO-for-HAL`

- **Keep your branch up to date:** Before merging or integrating changes, fetch the latest changes from `origin/main` and merge `origin/main` into your local branch to account for recent changes and minimize merge conflicts.

## 5. Opening a Pull Request (PR)

- **Do final Sync with `main`:** Prior to creating a PR, fetch the latest changes from `origin/main` and merge them into your local branch to avoid reviewers needing to resolve merge conflicts.
- **Follow PR guidelines:** Refer to [Writing and Reviewing a PR](/docs/procedure/Writing_And_Reviewing_A_PR.md) for guidelines on writing a descriptive PR title, a clear description, and reproducible testing steps.
- **Notify the team:** Post a message containing the PR link in the `PR Review Pings` thread in the Software channel on Discord.
- **Check the review thread regularly:** Team members should frequently check this thread for open PRs and review requests to help keep development moving.

## 6. Once Your PR Has been Approved

- **Squash and Merge:** The best part! Now that your PR has been reviewed and approved, you get to press the green button on the PR that says "squash and merge" and your work get merged with main.
- **Add Documentation:** Please navigate to the docs/technical folder and add documentation to the file that corresponds with the work you just completed.
- **Delete the Branch:** you will no longer be needing the branch, so once the PR has been approved and merged, delete it. You can do with in the github GUI by navigating to the closed PR and clicking into it, near the bottom
 of the PR a "delete branch" button will have appeared. To remove references to deleted remote branches on your local version simply run: `git fetch --prune`.