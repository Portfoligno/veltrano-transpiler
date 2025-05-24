# Veltrano Transpiler Development Guidelines

## Session Start Protocol - CRITICAL
- **FIRST ACTION:** Always read `WORKSPACE.md` using Read tool to load project context and memory

## File Formatting - CRITICAL RULES
- **EVERY file MUST end with a trailing newline (empty line)**
- All files MUST be formatted with the appropriate code formatter

### Trailing Newline Enforcement
**TOOL LIMITATION:** The `Write` and `Edit` tools cannot add trailing newlines directly.

**WORKAROUND PROCESS:**
1. **After using `Write` or `Edit` tools:** Always check with `Read` tool first to see current newline status
2. **Add newline only if missing:** If content ends immediately without empty line, then add:
   ```bash
   echo "" >> filename
   ```
3. **Visual indicators in `Read` tool:**
   - Missing newline: Content ends immediately (e.g., "line 5    some text")
   - Has newline: Shows empty line after content (e.g., "line 5    some text\nline 6")
   - Double newlines: Shows two empty lines (avoid this!)
4. **Before committing:** Always verify with `Read` tool and fix if needed
5. **Verification command:** Use this to check files:
   ```bash
   find . -name "*.rs" -o -name "*.vl" -o -name "*.md" -o -name "*.toml" | xargs -I {} sh -c 'if [ ! -s "{}" ] || [ "$(tail -c1 "{}" | wc -l)" -eq 0 ]; then echo "Missing trailing newline: {}"; fi'
   ```

**MANDATORY STEPS:**
- After creating/editing ANY file with `Write`/`Edit`: Use `Read` tool to check, then add newline only if missing
- **NEVER** blindly run `echo "" >> filename` without checking first - this creates duplicate newlines
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
- Use `git push --no-progress` to suppress progress indicators while keeping push summary
- Alternative: `git push --quiet` for minimal output (errors only)
- **After every push:** Update `WORKSPACE.md` with details of the work completed

## Memory Management

### WORKSPACE.md File
- **Purpose:** Store project context and memory across Claude Code sessions
- **Location:** `WORKSPACE.md` (gitignored for privacy)
- **Usage:** Read at session start, update throughout work
- **Token Limit:** Maximum 400 tokens (~300 words) to maintain readability and context efficiency

### Memory Guidelines
1. **Session Start:** Always read `WORKSPACE.md` to understand project context
2. **During Work:** Update memory with significant discoveries or changes
3. **Important Context:** Store key architectural decisions, patterns, and conventions
4. **Recent Work:** Document completed tasks and current project state
5. **MANDATORY:** After completing any significant work (bug fixes, features, refactoring), MUST update `WORKSPACE.md` before ending session
6. **Token Management:** If WORKSPACE.md exceeds 400 tokens, archive older "Recent Work" entries and keep only the most relevant context

### What to Store in Memory
- Project purpose and architecture overview
- Key file locations and their purposes
- Development patterns and conventions used
- Recent significant changes or refactoring (keep latest 3-5 major items)
- Important gotchas or special considerations
- Current project status and priorities

### Memory Maintenance
When WORKSPACE.md approaches 400 tokens:
1. **Archive Strategy:** Move older "Recent Work" entries to end of file under "## Archive" section
2. **Prioritize Recency:** Keep most recent 3-5 significant work items in main "Recent Work" section
3. **Preserve Core Context:** Never remove Project Context, Key Files, or Development Guidelines
4. **Rotation:** Archive entries older than 10-15 commits or 2-3 weeks of active development
