# Veltrano Transpiler Development Guidelines

## Session Start Protocol - CRITICAL
- **FIRST ACTION:** Always read `WORKSPACE.md` using Read tool to load project context and memory

## File Formatting - CRITICAL RULES
- **EVERY file MUST end with a trailing newline (empty line)**
- All files MUST be formatted with the appropriate code formatter

### Rust Code Formatting
- **Tool:** Use `cargo fmt` to format all Rust files
- **When to run:** Before every commit, after making code changes
- **Command:** `cargo fmt` (formats entire project)
- **Check formatting:** `cargo fmt --check` (verifies without modifying)

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
2. **Format code:** Run `cargo fmt` to ensure consistent formatting
3. **Verify formatting:** Check that new/edited files end with trailing newlines
4. **Stage + Commit:** Single tool call combining `git add` and `git commit`
5. **Push:** Run `git push` (no summary output required)

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
2. **During Work:** Update `WORKSPACE.md` immediately after completing any significant work or discoveries
3. **Update Triggers:** After fixing bugs, adding features, refactoring, or discovering important patterns/gotchas

### Value-Driven Update Strategy

**Goal:** Store information that maximizes context reuse for future sessions, regardless of format.

#### **Update Immediately After:**
- **Completing any significant work** - bug fixes, features, refactoring
- **Making important discoveries** - architectural insights, gotchas, patterns
- **Solving problems** - debugging breakthroughs, build issues, testing approaches
- **Major file changes** - new files, moved files, significant restructuring

#### **High-Value Information (Always Keep)**
- **Current project state** - what works, what's broken, active issues
- **Recent significant changes** - with enough detail for future context
- **Critical discoveries** - gotchas, performance insights, debugging tips
- **Active development context** - files being worked on, testing approaches

#### **Flexible Content Strategy**
- **Format freedom:** Use whatever structure best conveys the information
- **Context over convention:** Prioritize useful content over maintaining sections
- **Optimize for handoff:** Focus on "What would I need to know to continue this work?"
- **Token management:** When approaching 400 tokens, remove least valuable content first

#### **Maintenance Process**
- **Immediate updates:** Don't wait for session end - update as you work
- **Token pruning:** Remove old/less-relevant content when space is needed
- **Value assessment:** Keep information that provides ongoing context, remove completed work that won't help future sessions

