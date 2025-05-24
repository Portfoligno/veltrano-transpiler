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
- **After every push:** Update `MEMORY.md` with details of the work completed

## Memory Management

### MEMORY.md File
- **Purpose:** Store project context and memory across Claude Code sessions
- **Location:** `/MEMORY.md` (gitignored for privacy)
- **Usage:** Read at session start, update throughout work
- **Token Limit:** Maximum 800 tokens (~600 words) to maintain readability and context efficiency

### Memory Guidelines
1. **Session Start:** Always read `MEMORY.md` to understand project context
2. **During Work:** Update memory with significant discoveries or changes
3. **Important Context:** Store key architectural decisions, patterns, and conventions
4. **Recent Work:** Document completed tasks and current project state
5. **MANDATORY:** After completing any significant work (bug fixes, features, refactoring), MUST update `MEMORY.md` before ending session
6. **Token Management:** If MEMORY.md exceeds 800 tokens, archive older "Recent Work" entries and keep only the most relevant context

### What to Store in Memory
- Project purpose and architecture overview
- Key file locations and their purposes
- Development patterns and conventions used
- Recent significant changes or refactoring (keep latest 3-5 major items)
- Important gotchas or special considerations
- Current project status and priorities

### Memory Maintenance
When MEMORY.md approaches 800 tokens:
1. **Archive Strategy:** Move older "Recent Work" entries to end of file under "## Archive" section
2. **Prioritize Recency:** Keep most recent 3-5 significant work items in main "Recent Work" section
3. **Preserve Core Context:** Never remove Project Context, Key Files, or Development Guidelines
4. **Rotation:** Archive entries older than 10-15 commits or 2-3 weeks of active development


