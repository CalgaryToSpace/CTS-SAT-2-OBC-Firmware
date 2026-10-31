# How to Write a Pull Request (PR)

## Intro

A good PR should make it easy for a reviewer to understand **what changed, why it changed, and how to verify it**.

1. Give the PR a **descriptive title** that clearly explains what the PR accomplishes. Avoid vague titles like `fix stuff` or `updates`.
2. Write a **short description of what the PR achieves**. Explain the problem being addressed and the resulting change.
3. Include **testing steps** that a reviewer can follow to reproduce and verify the change on their own machine. Include commands, inputs, expected outputs, or other relevant setup.
4. If there is a **Github issue** that this PR will close/resolve, link it under the Development section.
5. Make sure **CI/checks pass** before requesting review.

### PR Description

At minimum, a PR description should answer:

* **What does this PR achieve?**
* **How can a reviewer test it?**

For example:

```markdown
## What

This PR implements a new telecommand that returns the current Unix timestamp. Had to work with Orbit to get datasheets and information about the GNSS, which is able to retreive accurate timestamp information. 

## Testing

1. Ensure you have SerialTest open and can ping the hello_world() command and get a response.
2. Check out to my branch, build and flash the board.
3. Now try to send the get_unix_timestamp() command and put that in a Unix timestamp converter to ensure it's close.
```

# How to Review a Pull Request (PR)

## Intro

We review code to make sure it's right, 

1. You don't have to be more experienced that the person who submitted code to review their code. Anyone can review anyone's code.
2. Plan to focus on the code for 20 minutes. Sometimes reviews take less time, but allow yourself the time you need to fully read the proposed code.
3. As you review code, treat it as a learning opportunity. People who read more code write better code.

## Review Questions (things to consider)

When reviewing a PR, consider the following questions:

1. Is the branch up-to-date (rebased), and do all checks pass (CI)? If not, it may be best to hold off on reviewing.
2. Does the code make sense? If not, is it because it's structured badly, documented badly, needs more explanation in the PR/in the reason for the code?
3. Is the code the right "complexity"? Does each function do one clear thing? Should any functions be split apart and/or joined together?
4. Is there repeated code (either repeated in the same PR, or repeated elsewhere in the repo)? Do we need to apply the DRY (don't repeat yourself) principle?
5. Do function names and variables describe what the functions do, in a global context? For example "uart_buffer" is a horrible name; "mpi_uart_buffer" is far better.
6. Provide at least one review comment like "wow this is a really nice/clear/clean implementation" or "great idea in this part". Reinforce the positives; code reviews are mentally tough to read, and it's nice stumbling upon these happy comments.
7. Are there adequate tests to support the code and ensure it can be maintained/modified in the future? Do the tests actually test the useful behaviours, or are they AI slop? Are there parts of untestable functions that should be split apart into testable units?

Leave comments/code review suggestions on the PR as you go.
