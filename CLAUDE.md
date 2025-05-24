# Veltrano Transpiler Development Guidelines

## File Formatting - CRITICAL RULES
- **EVERY file MUST end with a trailing newline (empty line)**
- All files MUST be formatted with the appropriate code formatter

### Trailing Newline Enforcement
**BEFORE creating or editing ANY file:**
1. When using `Write` tool: ALWAYS add `\n` at the end of content
2. When using `Edit` tool: Ensure new_string ends with newline if it's the end of file
3. When creating new files: ALWAYS verify they end with trailing newline

**BEFORE committing:**
- Use `Read` tool to verify newly created/edited files end with trailing newline
- If missing, fix immediately before staging

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
2. **Verify formatting:** Check that new/edited files end with trailing newlines
3. **Stage + Commit:** Single tool call combining `git add` and `git commit`
4. **Push:** Run `git push` (no summary output required)

### Push Behavior
- Do NOT show summary after `git push`
