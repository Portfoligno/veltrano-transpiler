# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Table of Contents
- [🚨 Critical Protocols](#-critical-protocols)
- [📂 Memory Management](#-memory-management)
- [🔧 Development Workflow](#-development-workflow)
- [🚀 Release Process](#-release-process)
- [⚠️ Common Pitfalls](#️-common-pitfalls)

---

## 🚨 Critical Protocols

### Session Start Protocol - MANDATORY
- **FIRST ACTION:** Always read `WORKSPACE.md` using Read tool to load project context and memory
  - This MUST happen before ANY other action, even if the user gives a specific request
  - NO EXCEPTIONS: Even for "quick" tasks or urgent requests
  - If you haven't read WORKSPACE.md yet, stop and read it first
  - **ENFORCEMENT:** Start every response by checking if you've read WORKSPACE.md
  - **VIOLATION CONSEQUENCE:** Session restart required if WORKSPACE.md not read first

### File Formatting Requirements
- **EVERY Git-tracked file MUST end with a trailing newline (empty line)**
  - Gitignored files do not require trailing newline checks
- All files MUST be formatted with the appropriate code formatter
- **Run `cargo fmt` before committing Rust code changes**

---

## 📂 Memory Management

### WORKSPACE.md File
- **Purpose:** Store project context and memory across Claude Code sessions
- **Location:** `WORKSPACE.md` (gitignored for privacy)
- **Usage:** Read at session start, update throughout work
- **Token Limit:** Maximum 400 tokens (~300 words) to maintain readability and context efficiency

### Memory Guidelines
1. **Session Start:** Always read `WORKSPACE.md` to understand project context
2. **During Work:** Update `WORKSPACE.md` immediately after significant discoveries or architectural insights
3. **Update Triggers:** After discovering important patterns/gotchas, debugging breakthroughs, or major architectural changes
4. **Before Major Actions:** Re-read WORKSPACE.md if you realize you haven't loaded context yet
5. **Focus:** Prioritize current project state, TODOs, and critical insights over completed work history

### MANDATORY Update Checkpoints
- **After discovering syntax limitations:** Document unsupported language features immediately
- **After creating new examples:** Update "Recent Session Work" with examples created
- **After major debugging sessions:** Document solutions and gotchas discovered
- **Before committing changes:** Update WORKSPACE.md with session summary
- **Every 3-4 tool uses:** Check if WORKSPACE.md needs updates

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
- **Making important discoveries** - architectural insights, gotchas, patterns
- **Solving complex problems** - debugging breakthroughs, build issues, testing approaches
- **Major architectural changes** - significant restructuring, design decisions
- **Finding critical insights** - performance patterns, development gotchas

#### **High-Value Information (Always Keep)**
- **Current project state** - what works, what's broken, active issues
- **TODOs and ongoing work** - tasks in progress, blockers, next steps
- **Critical discoveries** - gotchas, performance insights, debugging tips
- **Active development context** - key architecture decisions, design patterns

#### **Flexible Content Strategy**
- **Format freedom:** Use whatever structure best conveys the information
- **Context over convention:** Prioritize useful content over maintaining sections
- **Optimize for handoff:** Focus on "What would I need to know to continue this work?"
- **Token management:** When approaching 400 tokens, remove least valuable content first

#### **Maintenance Process**
- **Selective updates:** Update when discoveries provide ongoing value for future sessions
- **Token pruning:** Remove old/less-relevant content when space is needed, prioritize TODOs and insights
- **Value assessment:** Keep information that provides ongoing context, focus on current state over history

---

## 🔧 Development Workflow

### Code Formatting

#### Rust Code Formatting
- **Tool:** Use `cargo fmt` to format all Rust files
- **When to run:** Before committing changes that include Rust files
- **Command:** `cargo fmt` (formats entire project)
- **Check formatting:** `cargo fmt --check` (verifies without modifying)

#### MANDATORY Pre-Commit Checklist
1. If Rust files (`.rs`) changed: Run `cargo fmt`
2. Check for any new/modified files with `git status`
3. If Rust code changed: Verify tests pass with `cargo test`
4. Only then proceed with commit

#### Trailing Newline Enforcement
**TOOL LIMITATION:** The `Write` and `Edit` tools cannot add trailing newlines directly.

**SCOPE:** This applies only to Git-tracked files. Gitignored files do not require trailing newline checks.

**WORKAROUND PROCESS:**
1. **After using `Write` or `Edit` tools:** Always check with `Read` tool first to see current newline status
2. **Add newline only if missing:** If content ends immediately without empty line, then add:
   ```bash
   echo "" >> filename
   ```
3. **Visual indicators in `Read` tool:**
   - **IMPORTANT:** The Read tool output can be misleading for trailing newlines
   - Missing newline: Last line shows content with no subsequent line number
   - Has newline: May not be visually obvious in Read tool output
   - **BEST PRACTICE:** Use git diff after edits to verify - if git shows no `\ No newline at end of file`, the file is correct
4. **Before committing:** Always verify with `Read` tool and fix if needed
5. **Verification command:** Use this to check Git-tracked files only:
   ```bash
   git ls-files '*.rs' '*.vl' '*.md' '*.toml' | xargs -I {} sh -c 'if [ ! -s "{}" ] || [ "$(tail -c1 "{}" | wc -l)" -eq 0 ]; then echo "Missing trailing newline: {}"; fi'
   ```

**WARNING about newline detection:**
- **NEVER use `tail -n X`** to check for trailing newlines - it only shows non-empty lines
- The Read tool output can be ambiguous - absence of a line after content doesn't always mean missing newline
- **MOST RELIABLE:** Check git diff after changes - Git will show `\ No newline at end of file` if truly missing
- **Alternative check:** `tail -c1 file | wc -l` returns 1 if newline exists, 0 if missing

**WARNING about cargo fmt output:**
- When cargo fmt shows "No newline at end of file" in system reminders, this can be misleading
- This message might appear even when the file HAS a trailing newline
- **ALWAYS verify with Read tool** before adding a newline - never trust the message alone
- Adding a newline when one already exists creates unwanted double newlines

**MANDATORY STEPS:**
- After creating/editing ANY file with `Write`/`Edit`: Check git diff to see if newline is missing
- **NEVER** blindly run `echo "" >> filename` without verifying it's actually missing
- **CAUTION:** The Read tool alone is not sufficient to determine newline presence
- Before any commit: Use git diff to check - Git will warn about missing newlines

### Git Workflow - CRITICAL RULES

#### Staging and Committing
❌ **NEVER** run `git add` as a separate command  
✅ **ALWAYS** combine staging and committing in ONE tool call

**Required Pattern:**
```bash
git add file1 file2 && git commit -m "message"
```

**OR use automatic staging (for modified files only):**
```bash
git commit -a -m "message"
```

**Special Cases:**
- **File renames:** `git add old_name new_name && git commit -m "message"`
- **File deletions:** `git add deleted_file && git commit -m "message"`
- **Multiple operations:** Stage all affected files together to preserve Git's rename detection

#### Commit Process
1. **Analyze changes:** Run `git status`, `git diff`, and `git log` in parallel
2. **FORMAT CODE - ONLY IF RUST FILES CHANGED:** 
   - Check if any `.rs` files were modified: `git diff --name-only | grep -q '\.rs$'`
   - If Rust files changed: Run `cargo fmt` to ensure consistent formatting
   - If only non-Rust files changed (e.g., `.md`, `.toml`): Skip cargo fmt
   - If formatter makes changes, include those in the commit
3. **Verify formatting:** Check that new/edited files end with trailing newlines
4. **Stage + Commit:** Single tool call combining `git add` and `git commit`
5. **Push:** Run `git push` (no summary output required)

#### Push Behavior
- Use `git push --no-progress` to suppress progress indicators while keeping push summary
- Alternative: `git push --quiet` for minimal output (errors only)

---

## 🚀 Release Process

### Version Numbering Pattern
This project uses a development version with `-dev` suffix that gets released without the suffix:

1. **Development Version**: Work happens on version with `-dev` suffix (e.g., `0.1.2-dev`)
2. **Release Version**: When ready to release, remove `-dev` suffix (e.g., `0.1.2`)
3. **Next Development**: After tagging, bump to next version with `-dev` (e.g., `0.1.3-dev`)

The pattern repeats: develop on `X-dev`, release as `X`, then move to `(X+1)-dev`.

### CHANGELOG.md Guidelines

**CRITICAL: Timing and Scope**
1. **Only include changes between the previous release tag and the current release commit**
   - Changes made AFTER starting the release process belong in the NEXT release
   - Example: If preparing 0.2.2, only include commits since 0.2.1 tag up to the release commit
2. **Verify what existed in the previous release**
   - Use `git show <prev-tag>:src/ast.rs` to check if a feature existed before
   - Use `git log <prev-tag>..HEAD` to see all changes since last release

**What to include in CHANGELOG entries:**
- User-facing changes to the transpiler functionality
- New language features or syntax support (e.g., data classes, new operators)
- Changes to existing language behavior
- Bug fixes that affect transpiler output
- Breaking changes or deprecations
- New reserved keywords

**What NOT to include in CHANGELOG entries:**
- Internal refactoring that doesn't change user-visible behavior
- Changes to `CLAUDE.md` or development documentation
- Test additions/changes (unless they reveal fixed bugs)
- Code style/formatting changes
- Build process changes (unless they affect users)
- Internal implementation details (e.g., AST node renames, internal enum changes)

**Category Guidelines:**
- **Added**: Features that didn't exist in any form in the previous release
- **Changed**: Modifications to existing user-visible features
- **Fixed**: Bug fixes for issues that existed in the previous release
- **Deprecated**: Features marked for future removal
- **Removed**: Features that existed in previous release but are now gone
- **Security**: Security-related fixes

**Common Mistakes to Avoid:**
- ❌ Including uncommitted or future changes in the CHANGELOG
- ❌ Documenting internal refactoring as user-facing changes
- ❌ Listing a feature as "Changed" when it was newly added since last release
- ❌ Including iterative development steps instead of the final feature
- ❌ Documenting removed features that were never in a release

**Format:**
- Follow [Keep a Changelog](https://keepachangelog.com/) format
- Group changes under: Added, Changed, Fixed, Deprecated, Removed, Security
- Focus on what users need to know, not internal development details
- Be specific about what generates what (e.g., "MutRef(v) generates &mut (&v).clone()")

### Release Steps
When the user requests a release:

1. **Update version for release**
   - Remove `-dev` suffix from version in `Cargo.toml`
   - Run `cargo build` or `cargo check` to update `Cargo.lock`
   - Update or create `CHANGELOG.md` entry for the new version (following guidelines above)
   
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


---

## ⚠️ Common Pitfalls

### 1. Skipping WORKSPACE.md on Session Start
**Problem:** Jumping directly into user requests without loading context  
**Solution:** Make reading WORKSPACE.md a reflex action - do it before even thinking about the user's request  
**Self-Check:** "Have I read WORKSPACE.md yet?" - If no, stop everything and read it
**New Enforcement:** Every response must begin with explicit confirmation of WORKSPACE.md loading

### 2. Not Capturing Important Insights in WORKSPACE.md
**Problem:** Missing valuable discoveries that would help future sessions  
**Solution:** When you find important patterns, gotchas, or architectural insights, document them in WORKSPACE.md  
**Self-Check:** "Did I learn something that would help me (or another session) work on this project later?"
**New Triggers:** Document syntax limitations, successful examples, debugging solutions immediately

### 3. Not Using TODO Management
**Problem:** Losing track of ongoing work across sessions  
**Solution:** Actively maintain TODO section, marking items as IN PROGRESS or completed  
**Self-Check:** "Is my current task in the TODO list? Are completed items marked?"

### 4. Misinterpreting Trailing Newline Status
**Problem:** Adding unnecessary newlines based on misleading Read tool output  
**Solution:** Use git diff to check - Git explicitly shows `\ No newline at end of file` when missing  
**Self-Check:** "Does git diff show a newline warning? If not, the file is fine"

### 5. Failing to Update WORKSPACE.md During Extended Sessions
**Problem:** Completing significant work without documenting discoveries, syntax limitations, or new examples  
**Solution:** Set systematic checkpoints every few tool uses to assess if WORKSPACE.md needs updates  
**Self-Check:** "Have I discovered anything that would help future sessions? Have I created examples? Have I found limitations?"
**Triggers:** Any syntax error resolution, any successful example creation, any debugging breakthrough

---

## 📚 Additional Information

### Important Instructions
Codebase and user instructions are shown below. Be sure to adhere to these instructions. IMPORTANT: These instructions OVERRIDE any default behavior and you MUST follow them exactly as written.

### File Footer Requirements
**EVERY Git-tracked file MUST end with a trailing newline (empty line)**

