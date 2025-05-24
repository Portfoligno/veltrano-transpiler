# Changed File Rules
- Ensure each changed file ends with a trailing empty line
- Ensure each changed file is formatted with the code formatter

# Git Commit and Push Behavior
- ALWAYS combine git add and git commit in a single tool call - NEVER run git add separately
- Use parallel tool calls to run git add and git commit commands together
- Alternative: Use `git commit -a` to automatically stage and commit all modified files
- Do not show a summary after git push

## Git Workflow Reminder
When committing changes, follow this exact pattern:
1. Run git status, git diff, and git log in parallel to understand changes
2. Combine git add and git commit in a single tool call (or use git commit -a)
3. Push without showing summary
