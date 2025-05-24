# Veltrano Transpiler Development Guidelines

## File Formatting
- Each file MUST end with a trailing empty line
- All files MUST be formatted with the appropriate code formatter

## Git Workflow - CRITICAL RULES

### Staging and Committing
❌ **NEVER** run `git add` as a separate command  
✅ **ALWAYS** combine staging and committing in ONE tool call

**Required Pattern:**
```bash
git add file1 file2 && git commit -m "message"
```

**OR use automatic staging:**
```bash
git commit -a -m "message"
```

### Commit Process
1. **Analyze changes:** Run `git status`, `git diff`, and `git log` in parallel
2. **Stage + Commit:** Single tool call combining `git add` and `git commit`
3. **Push:** Run `git push` (no summary output required)

### Push Behavior
- Do NOT show summary after `git push`
