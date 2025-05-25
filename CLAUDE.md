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

## Release Process

### Version Numbering Pattern
This project uses a development version with `-dev` suffix that gets released without the suffix:

1. **Development Version**: Work happens on version with `-dev` suffix (e.g., `0.1.2-dev`)
2. **Release Version**: When ready to release, remove `-dev` suffix (e.g., `0.1.2`)
3. **Next Development**: After tagging, bump to next version with `-dev` (e.g., `0.1.3-dev`)

The pattern repeats: develop on `X-dev`, release as `X`, then move to `(X+1)-dev`.

### Release Steps
When the user requests a release:

1. **Update version for release**
   - Remove `-dev` suffix from version in `Cargo.toml`
   - Run `cargo build` or `cargo check` to update `Cargo.lock`
   - Update or create `CHANGELOG.md` entry for the new version
   
2. **Commit the release**
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md && git commit -m "Release X.Y.Z

   - Summary of major changes
   - Other important notes

   🤖 Generated with [Claude Code](https://claude.ai/code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

3. **Create annotated tag** (without 'v' prefix)
   ```bash
   git tag -a X.Y.Z -m "Release X.Y.Z
   
   Brief summary of release
   
   See CHANGELOG.md for full details."
   ```

4. **Push commits and tag**
   ```bash
   git push && git push origin X.Y.Z
   ```

5. **Begin next development cycle**
   - Bump version to next patch/minor/major with `-dev` suffix in `Cargo.toml`
   - Run `cargo build` or `cargo check` to update `Cargo.lock`
   - Commit: 
   ```bash
   git add Cargo.toml Cargo.lock && git commit -m "Begin X.Y.Z-dev development cycle

   🤖 Generated with [Claude Code](https://claude.ai/code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```
   - Push the development version

### Important Notes
- Version tags do NOT use 'v' prefix (use `0.1.2`, not `v0.1.2`)
- Development versions always use `-dev` suffix
- The version in Cargo.toml should match the release tag exactly when tagging
- Update WORKSPACE.md after pushing the release

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

### TODO Management in WORKSPACE.md
**Proactively maintain a TODO section** to track work across sessions:

1. **Automatic TODO Updates**
   - When starting a task: Mark as "IN PROGRESS"
   - When completing a task: Mark with `[x]` immediately
   - When discovering issues: Add new TODOs right away
   - When finding blockers: Document them with the TODO

2. **TODO Format:**
   ```markdown
   ## TODO
   ### Category Name
   - [x] Completed task
   - [ ] Pending task - IN PROGRESS (if actively working)
   - [ ] Future task (with context/reason)
   ```

3. **Best Practices**
   - Group related tasks under descriptive categories
   - Include error messages or line numbers for bugs
   - Note which files/examples are affected
   - Add "CRITICAL:" prefix for blocking issues
   - Keep TODOs actionable and specific

4. **Continuous Maintenance**
   - Don't wait for user to ask about TODOs
   - Update immediately as work progresses
   - Remove completed work only if no longer relevant
   - Preserve context for future sessions

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

