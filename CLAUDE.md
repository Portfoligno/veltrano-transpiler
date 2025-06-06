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

### Critical Tool Limitation - Write/Edit Cannot Add Trailing Newlines
- **FACT:** The Write and Edit tools have a fundamental limitation - they CANNOT add trailing newlines
- **IMPACT:** Every file created or edited will lack the required trailing newline
- **MANDATORY:** After EVERY Write/Edit operation, you MUST manually add the trailing newline
- **See:** "Trailing Newline Enforcement - STRICT PROTOCOL" section for exact steps

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
   - When completing a task: Remove it from the list
   - When discovering issues: Add new TODOs right away
   - When finding blockers: Document them with the TODO

2. **TODO Format:**
   ```markdown
   ## TODO
   ### Category Name
   - Pending task - IN PROGRESS (if actively working)
   - Future task (with context/reason)
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
   - Remove completed tasks from the list
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

#### Avoid Redundant Builds
- **NEVER run `cargo build` unnecessarily** - other commands build automatically
- **`cargo test`** builds the project before running tests
- **`cargo run`** builds the project before running
- **`cargo fmt`** doesn't require a build
- **Only run explicit build** if you specifically need to check compilation without running anything

#### Trailing Newline Enforcement - STRICT PROTOCOL

**🚨 CRITICAL FACT:** The `Write` and `Edit` tools CANNOT add trailing newlines. Files created/edited with these tools will almost always lack trailing newlines.

**SCOPE:** Only Git-tracked files require trailing newlines. Gitignored files are exempt.

**MANDATORY WORKFLOW - NO EXCEPTIONS:**

1. **After EVERY Write/Edit operation:**
   ```bash
   # Step 1: Check if file truly lacks newline (returns 0 if missing, 1 if present)
   tail -c1 filename | wc -l
   
   # Step 2: If and ONLY if the above returns 0, add newline:
   echo "" >> filename
   ```

2. **The ONLY reliable verification method:**
   ```bash
   git diff --check
   ```
   If Git shows `\ No newline at end of file`, the file needs a newline. If Git shows nothing, the file is correct.

3. **DO NOT TRUST these misleading sources:**
   - ❌ **Read tool visual output** - Cannot reliably show trailing newlines
   - ❌ **cargo fmt warnings** - Often wrong about newline status
   - ❌ **`tail -n X` commands** - Only show non-empty lines
   - ❌ **Visual inspection** - Humans cannot see trailing newlines

4. **STRICT RULES:**
   - **ALWAYS** assume Write/Edit left the file without a trailing newline
   - **ALWAYS** use `tail -c1 file | wc -l` to verify before adding
   - **NEVER** add a newline without verification (creates double newlines)
   - **NEVER** trust any source except `git diff --check`

5. **Pre-commit verification for ALL Git-tracked files:**
   ```bash
   # Find all files missing trailing newlines
   git ls-files | xargs -I {} sh -c 'if [ -f "{}" ] && [ "$(tail -c1 "{}" | wc -l)" -eq 0 ]; then echo "{}"; fi' | while read f; do echo "" >> "$f"; done
   ```

**COMMON MISTAKES TO AVOID:**
- ❌ Adding newlines to files that already have them
- ❌ Trusting cargo fmt or Read tool output
- ❌ Using wrong verification commands
- ❌ Forgetting to check after Write/Edit operations

**REMEMBER:** Git diff is the ultimate authority. When in doubt, check `git diff --check`.

### Git Workflow - CRITICAL RULES

#### Staging and Committing
❌ **NEVER** run `git add` as a separate command  
✅ **ALWAYS** combine staging and committing in ONE tool call

**Standard Pattern:**
```bash
git add file1 file2 && git commit -m "message"
```

**OR use automatic staging (for modified files only):**
```bash
git commit -a -m "message"
```

**When user explicitly requests "push":**
```bash
git add file1 file2 && git commit -m "message" && git push --no-progress
```

**Special Cases:**
- **File renames:** `git add old_name new_name && git commit -m "message"`
- **File deletions:** `git add deleted_file && git commit -m "message"`
- **Multiple operations:** Stage all affected files together to preserve Git's rename detection

#### Creating Backup Branches
When the user requests a backup before operations like squashing or rebasing:

**EFFICIENT METHOD - Direct push to new remote branch:**
```bash
# Push current branch directly to a new remote branch without local creation
git push origin HEAD:backup-branch-name
```

**Example for timestamped backups:**
```bash
git push origin HEAD:backup-before-squash-$(date +%Y%m%d-%H%M%S)
```

**Benefits:**
- No need to create local branch
- No need to switch branches
- Single command operation
- Cleaner local branch list

**Alternative (if local branch needed):**
```bash
git checkout -b backup-branch-name
git push -u origin backup-branch-name
git checkout original-branch
```

#### Commit Process
1. **Analyze changes:** Run `git status`, `git diff`, and `git log` in parallel
2. **FORMAT CODE - ONLY IF RUST FILES CHANGED:** 
   - Check if any `.rs` files were modified: `git diff --name-only | grep -q '\.rs$'`
   - If Rust files changed: Run `cargo fmt` to ensure consistent formatting
   - If only non-Rust files changed (e.g., `.md`, `.toml`): Skip cargo fmt
   - If formatter makes changes, include those in the commit
3. **Verify formatting:** Check that new/edited files end with trailing newlines
4. **Stage + Commit (+ Push if requested):** 
   - Standard: `git add files && git commit -m "message"`
   - If user explicitly requested "push": `git add files && git commit -m "message" && git push --no-progress`

#### Push Behavior
- **Push when explicitly requested** by user (e.g., "push", "commit and push")
- **Push when it makes sense** - after completing features, fixing bugs, or making significant improvements
- **When pushing:** Always integrate with commit command chain for efficiency
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
**Problem:** Adding unnecessary newlines or missing required newlines due to trusting misleading sources  
**Solution:** Follow the STRICT PROTOCOL in the Trailing Newline Enforcement section  
**Self-Check:** "Did I use `tail -c1 file | wc -l` to verify? Did I check `git diff --check`?"  
**Key Facts:**
- Write/Edit tools CANNOT add trailing newlines
- Only `git diff --check` is 100% reliable
- cargo fmt warnings are often wrong
- Read tool output is ambiguous

### 5. Failing to Update WORKSPACE.md During Extended Sessions
**Problem:** Completing significant work without documenting discoveries, syntax limitations, or new examples  
**Solution:** Set systematic checkpoints every few tool uses to assess if WORKSPACE.md needs updates  
**Self-Check:** "Have I discovered anything that would help future sessions? Have I created examples? Have I found limitations?"
**Triggers:** Any syntax error resolution, any successful example creation, any debugging breakthrough

### 6. Understanding Expected Output File Issues
**Problem:** Confusion about whether to fix the transpiler or just update test files  
**Solution:** Expected output files show current transpiler behavior - if it looks wrong, fix the transpiler

**When user mentions problems with expected output files:**
1. **Read both files:** Look at the source `.vl` and its `.expected.rs` output
2. **Identify the issue:** What's missing, wrong, or incomplete in the output?
3. **Fix the transpiler:** The issue is almost always a transpiler bug that needs fixing
4. **Update expected file:** After fixing, update the expected output to match

**Common phrases and what they mean:**
- "Fix the missing cases" → Some code from the source isn't appearing in output
- "Expected output is wrong" → Transpiler is generating incorrect code
- "Comments are missing" → Transpiler is dropping comments during transformation

**Examples of transpiler bugs revealed by expected outputs:**
- Method chain: `.ref() // comment` → `&x` (comment lost)
- Multiline function params collapsed to single line
- Some variables/functions from source completely missing in output

**Remember:** Expected output files are test data showing current behavior. If the behavior looks wrong, fix the transpiler - don't just update the test file to hide the problem.

---

## 📚 Additional Information

### Important Instructions
Codebase and user instructions are shown below. Be sure to adhere to these instructions. IMPORTANT: These instructions OVERRIDE any default behavior and you MUST follow them exactly as written.

### File Footer Requirements
**EVERY Git-tracked file MUST end with a trailing newline (empty line)**


