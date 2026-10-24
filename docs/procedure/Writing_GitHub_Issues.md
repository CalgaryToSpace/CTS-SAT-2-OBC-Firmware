# How to Write a GitHub Issue

## Intro

A good issue should makes it easy to track work and it's progress.

1. Give the issue a **descriptive title** that clearly explains the task. 
2. Write a **minimum viable description** of what the issue should achieve. Focus on the desired outcome rather than prescribing every implementation detail. That will be the job of whoever picks up the task. Some level of investigation is always needed.
3. Ensure the issue represents a **single, testable unit of work** within a specific subsystem. Break larger tasks into smaller issues when necessary. In addition, a good sanity check for a good GitHub issue is being able to envision the unit tests required.
4. Verify that **all dependencies and prerequisites are already developed or available** before creating the issue. Issues should be scoped so they can be completed without relying on unfinished work or introducing unnecessary scope creep. If an issue is blocked, mark the issue as 'blocked by' in the Relationships section of a GitHub issue.
5. Add a **subsystem label** to identify the part of the codebase the issue relates to, such as `GNSS` or `EPS`.

## Before Creating an Issue

- [ ] The title clearly describes the task.
- [ ] The description defines the minimum required outcome.
- [ ] The issue is a single, testable unit of work.
- [ ] All required dependencies and prerequisites are available.
- [ ] The issue has the appropriate subsystem label linked(e.g., `GNSS`, `EPS`).
- [ ] All Issue Relationships have been accounted for (Blocked by, Related to, ect).