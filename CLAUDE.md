# Veltrano Transpiler Development Guidelines

## File Formatting - CRITICAL RULES
- **EVERY file MUST end with a trailing newline (empty line)**
- All files MUST be formatted with the appropriate code formatter

### Trailing Newline Enforcement
**TOOL LIMITATION:** The `Write` and `Edit` tools cannot add trailing newlines directly.

**WORKAROUND PROCESS:**
1. **After using `Write` or `Edit` tools:** Always use bash to add trailing newline:
   ```bash
   echo "" >> filename
   ```
2. **Before committing:** Always verify with `Read` tool and fix if needed
3. **Verification command:** Use this to check files:
   ```bash
   find . -name "*.rs" -o -name "*.vl" -o -name "*.md" -o -name "*.toml" | xargs -I {} sh -c 'if [ ! -s "{}" ] || [ "$(tail -c1 "{}" | wc -l)" -eq 0 ]; then echo "Missing trailing newline: {}"; fi'
   ```

**MANDATORY STEPS:**
- After creating/editing ANY file with `Write`/`Edit`: Run `echo "" >> filename`
- Before any commit: Run verification command and fix all missing newlines

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

## Memory Management

### MEMORY.md File
- **Purpose:** Store project context and memory across Claude Code sessions
- **Location:** `/MEMORY.md` (gitignored for privacy)
- **Usage:** Read at session start, update throughout work

### Memory Guidelines
1. **Session Start:** Always read `MEMORY.md` to understand project context
2. **During Work:** Update memory with significant discoveries or changes
3. **Important Context:** Store key architectural decisions, patterns, and conventions
4. **Recent Work:** Document completed tasks and current project state

### What to Store in Memory
- Project purpose and architecture overview
- Key file locations and their purposes
- Development patterns and conventions used
- Recent significant changes or refactoring
- Important gotchas or special considerations
- Current project status and priorities
