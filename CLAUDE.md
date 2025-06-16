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
- **FIRST ACTION:** Always attempt to read `WORKSPACE.md` using Read tool to load project context and memory
  - This MUST happen before ANY other action, even if the user gives a specific request
  - NO EXCEPTIONS: Even for "quick" tasks or urgent requests
  - **If WORKSPACE.md exists:** Read it completely and use the context
  - **If WORKSPACE.md doesn't exist:** Create it immediately with basic project structure
  - **ENFORCEMENT:** Start every response by checking if you've loaded/created WORKSPACE.md
  - **VIOLATION CONSEQUENCE:** Session restart required if WORKSPACE.md not handled first

### WORKSPACE.md Creation Protocol (when missing)
When WORKSPACE.md doesn't exist, create it immediately with:
1. **Project identification** (name, version, current branch)
2. **Empty TODO section** ready for use
3. **Architecture Notes section** for discoveries
4. **Development Gotchas section** for important patterns
5. **Current Project State** placeholder

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

### WORKSPACE.md Overview
- **Purpose:** Store project context across Claude Code sessions
- **Location:** `WORKSPACE.md` (gitignored)
- **Token Limit:** 400 tokens (~300 words)
- **Check Frequency:** Every 5-10 tool uses

### Usage Protocol
1. **Session Start:** ALWAYS read WORKSPACE.md first
2. **During Work:** Update after major discoveries or changes
3. **Token Check:** Run `wc -w WORKSPACE.md` regularly
4. **Over 300 words:** Prune immediately

### Anti-Violation Practices
**BEFORE adding anything new:**
1. **Remove something old first** - Practice "one in, one out"
2. **Resist completion tracking** - Your work is in git history, not WORKSPACE.md
3. **Challenge every line** - Ask "Will I need this tomorrow?" If unsure, delete it
4. **Convert to action** - Transform "We discovered X" into "TODO: Handle X"

**Pruning mindset:**
- **WORKSPACE.md is a sticky note, not a journal**
- **Information has negative value after 48 hours**
- **If you can't remember it, it wasn't important**
- **Verbose description = cognitive overload for future you**

### What to Track
**KEEP (High Value):**
- Active TODOs with context
- Current blockers/issues
- Recent discoveries (1-2 sessions)
- Critical architecture decisions
- Active development gotchas

**REMOVE (Low Value):**
- Completed tasks (including TODO items marked done)
- Historical narratives ("how we got here")
- Solved problems
- Old session summaries (keep only last 1-2)
- Verbose explanations

### TODO Management
- Mark active task as "IN PROGRESS"
- Remove completed items immediately
- Group by category with context
- Add "CRITICAL:" for blockers
- Keep actionable and specific

### Psychological Traps to Avoid
1. **"But I might need this later"** → You won't. Delete it.
2. **"This shows important progress"** → Git shows progress. Delete it.
3. **"Future me will appreciate the context"** → Future you will appreciate brevity. Delete it.
4. **"This was hard to figure out"** → Document it in code comments, not here. Delete it.
5. **"Just this once, I'll keep it"** → This is how hoarding starts. Delete it.

### Formatting Rules
- Bullet points, not paragraphs
- No emoji checkmarks (✅, ❌, etc.)
- Remove filler words
- Use clear abbreviations
- Focus on "what" not "why"

### Pruning Priority
When over token limit, remove in order:
1. Completed work descriptions
2. Old session summaries
3. Historical problem descriptions
4. Duplicate information
5. Verbose explanations (condense to bullets)

---

## 🔧 Development Workflow

### Code Quality Principles

#### Embrace Complexity as Value
- **Complexity in well-organized code is not a burden - it's accumulated value**
- **Each layer of complexity represents features that users need:**
  - AST complexity enables accurate source-to-source translation
  - Comment preservation complexity maintains developer intent
  - Parser complexity handles real-world code patterns
- **Building on existing complexity creates compounded value:**
  - We're not "adding complexity" negatively - we're building features
  - The complexity IS the product - it's what makes the transpiler useful
  - A "simple" transpiler that drops features would be easy but not valuable
- **Mindset: "I get to extend this system to deliver more value"**
  - Not: "Oh no, I have to update multiple files"
  - But: "Great, I can leverage this architecture to solve real problems"
  - The alternative (avoiding complexity) means avoiding usefulness
- **This transpiler's value comes from handling the complex cases correctly**

#### Use Precise Language When Identifying Issues
- **Avoid vague terms** like "incorrect", "wrong", or "broken" without explanation
- **Be specific about what you observe** vs what you expect
- **Don't overstate problems** - e.g., "excessive indentation" when the real issue is that inline comments are moved to separate lines
- **Focus on root causes** rather than symptoms:
  - ❌ "The indentation is wrong"
  - ✅ "Inline comments are being parsed as separate statements, causing them to appear on their own lines"
- **Verify your understanding** before proposing solutions:
  - Check if the output is actually correct before assuming it's wrong
  - Understand the existing behavior before calling it a "problem"
- **When debugging:**
  - State observations: "The comment appears on line X instead of line Y"
  - Identify the mechanism: "Comments are parsed as statements, not attached to expressions"
  - Propose targeted fixes: "Remove the newline before EndOfLine comments during generation"

#### Prioritize Proper System Design Over Quick Fixes
- **Always implement the correct architecture first** - Avoid hardcoding data that should be dynamically queried
- **Resist shortcuts that create technical debt** - Quick fixes often become permanent problems
- **When facing complex system integrations:**
  - First understand what the ideal solution would be
  - Research available tools, APIs, and existing infrastructure
  - Design the system to be extensible and maintainable
  - Implement proper abstractions even if they take longer initially
- **Common anti-patterns to avoid:**
  - Hardcoding external API responses or metadata
  - Duplicating information that exists elsewhere in the system
  - Creating "temporary" workarounds without clear migration paths
  - Building rigid systems that can't adapt to new requirements
- **Better approaches:**
  - Design plugin-based architectures for extensibility
  - Query authoritative sources for dynamic information

#### Handling Systematic Issues - NO REFUSAL POLICY
- **When the user identifies hardcoded values or systematic issues:**
  - **DO NOT refuse to fix them properly** - Even if it requires significant changes
  - **DO NOT suggest band-aid solutions** - Like keeping `SourceLocation::new(1, 1)` with TODOs
  - **DO investigate the root cause** - Trace through the code to understand the issue
  - **DO implement the complete fix** - Even if it requires refactoring multiple files
- **If a proper fix seems complex:**
  - Analyze the full scope of the problem
  - Document findings if investigation is needed
  - Propose and implement the correct solution
  - Never settle for workarounds when the user wants a proper fix
- **Example:** If user says "fix SourceLocation::new(1, 1)" - trace where location info should come from and implement proper propagation, don't just change the values
  - Build proper abstraction layers between components
  - Create clear interfaces that can evolve over time
  - Document assumptions and future improvement paths

#### Fail-Fast Behavior
- **Prefer explicit panics over silent failures** - when code reaches an impossible state
- **Use `panic!()` for "should never happen" scenarios** rather than fallback logic
- **Example cases for panics:**
  - Type system inconsistencies (unexpected type constructors)
  - Invalid invariants (Own<T> used with naturally owned types)
  - Unreachable code paths that indicate bugs
- **Benefits:** Catches bugs early, makes assumptions explicit, prevents silent corruption
- **When NOT to panic:** User input validation, external API failures, recoverable errors

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
   
   **CORRECT EXAMPLE:**
   ```bash
   # After editing tests/transpiler_integration.rs
   tail -c1 tests/transpiler_integration.rs | wc -l
   # If output is 0: echo "" >> tests/transpiler_integration.rs
   # If output is 1: DO NOTHING - file already has trailing newline
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
- ❌ **Running `echo "" >> filename` without checking first** - This adds ANOTHER newline, not ensures exactly one
- ❌ **Assuming every edit removes the trailing newline** - Many edits preserve existing newlines

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
- **File renames/moves:** Always include BOTH the old and new paths
  - **Best practice:** Use `git mv old_name new_name` - stages both automatically
  - **If using regular `mv`:** `git add old_name new_name && git commit -m "message"`
  - **Common scenarios:**
    - Moving files: `git mv old/path/file new/path/file`
    - Renaming files: `git mv old_name.ext new_name.ext`
    - Re-enabling examples: `git mv file.vl.disabled file.vl`
  - **If already used `mv`:** Check `git status` before committing to ensure both changes are staged
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

#### Understanding Context When User Reports Git Actions
- **When user mentions they've done something with git** (amended, rebased, cherry-picked, etc.):
  - **Don't assume** - verify the actual repository state
  - **Check what happened** using git commands:
    - `git status` - current working directory state
    - `git log --oneline -n 5` - recent commits
    - `git diff --stat HEAD~1` - what changed in recent commits
  - **Understand the implications** before taking further git actions
  - **Example:** "I've amended X" could mean:
    - Changes are already incorporated in an existing commit
    - Local history has diverged from remote
    - Simple pull/push may not work as expected
- **This prevents:** accidentally undoing user's deliberate git operations

#### Interactive Rebase Limitations
❌ **NEVER use `git rebase -i`** - Interactive rebase requires user input which is not available in Claude Code environment. The command will appear to succeed but won't actually squash commits.

#### Combining Commits - Basic Pattern
When the user requests combining commits (e.g., "combine the re-enabling with the fix commit" or "amend X onto Y"):

**CRITICAL UNDERSTANDING - "Amend X onto Y":**
- **Means**: Take commit X and merge it INTO commit Y
- **NOT**: Replay all commits after Y in order
- **KEY**: Cherry-pick X directly after resetting to Y, regardless of what's between them

**Standard Procedure:**
1. **Check current state and create backup**
   ```bash
   git status && git log --oneline -n 10
   git push origin HEAD:backup-$(date +%Y%m%d-%H%M%S)
   ```

2. **Hard reset to the base commit** (the one you want to keep)
   ```bash
   git reset --hard <base-commit-hash>
   ```

3. **Cherry-pick and amend the SPECIFIC commit** (not intermediate commits)
   ```bash
   git cherry-pick <commit-to-combine> && git reset --soft HEAD~1 && git commit --amend -m "Updated message reflecting combined changes"
   ```

4. **Cherry-pick any intermediate/subsequent commits**
   ```bash
   git cherry-pick <first-commit>^..<last-commit>
   ```

**Example - Combining C into A (where history is A→B→C):**
```bash
# WRONG: Don't cherry-pick intermediate commits first
git reset --hard A
git cherry-pick B  # WRONG!
git cherry-pick C

# CORRECT: Cherry-pick the target commit directly
git reset --hard A
git cherry-pick C         # Pick C directly
git reset --soft HEAD~1
git commit --amend -m "A with C's changes combined"
git cherry-pick B         # Then apply B on top
```

**Important Notes:**
- This rewrites history - requires force push if already pushed
- Make a backup branch before rewriting: `git push origin HEAD:backup-name`
- Use `git cherry-pick --abort` if something goes wrong during cherry-pick


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

### 6. Missing Files in Commits After Context Compaction
**Problem:** Multi-file operations (type migrations, refactoring) often leave modified files uncommitted, especially after session continuation  
**Solution:** Before any commit, verify all work is complete by checking file categories affected by your operation type  
**Self-Check:** "What type of operation was I doing? Which file categories should this have touched?"  
**Common Patterns:**
- **Type changes:** type_checker.rs → parser.rs → codegen.rs → rust_interop.rs → tests → examples
- **Language features:** ast.rs → parser.rs → codegen.rs → examples → tests  
- **Documentation:** README.md + related example files

### 7. Understanding Expected Output File Issues
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

### 8. Taking Responsibility When Breaking Things
**Problem:** Blaming "fundamental issues" or "pre-existing problems" when your changes cause test failures  
**Solution:** Take responsibility for breaking changes. If tests passed before and fail after your changes, you broke them.  
**Self-Check:** "Were these tests passing before my changes? Then I broke them."
**Correct Response:**
- Acknowledge the mistake immediately
- Understand what went wrong before trying again
- Don't blame the architecture or claim the tests were already broken

---

## 📚 Additional Information

### Important Instructions
Codebase and user instructions are shown below. Be sure to adhere to these instructions. IMPORTANT: These instructions OVERRIDE any default behavior and you MUST follow them exactly as written.

### File Footer Requirements
**EVERY Git-tracked file MUST end with a trailing newline (empty line)**


