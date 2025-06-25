# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Table of Contents
- [🚨 Session Management](#-session-management)
- [📂 WORKSPACE.md](#-workspacemd)
- [💻 Development Workflow](#-development-workflow)
- [📝 File Management](#-file-management)
- [🔀 Git Workflow](#-git-workflow)
- [🚀 Release Process](#-release-process)
- [⚠️ Common Pitfalls](#️-common-pitfalls)
- [📚 Project-Specific Notes](#-project-specific-notes)

---

## 🚨 Session Management

### Session Start Protocol - MANDATORY
- **FIRST ACTION:** Always attempt to read `WORKSPACE.md` using Read tool to load project context and memory
  - This MUST happen before ANY other action, even if the user gives a specific request
  - NO EXCEPTIONS: Even for "quick" tasks or urgent requests
  - **If WORKSPACE.md exists:** Read it completely and use the context
  - **If WORKSPACE.md doesn't exist:** Create it immediately with basic project structure
  - **ENFORCEMENT:** Start every response by checking if you've loaded/created WORKSPACE.md
  - **VIOLATION CONSEQUENCE:** Session restart required if WORKSPACE.md not handled first

### Understanding Two Different TODO Systems
**CRITICAL DISTINCTION - These are completely separate:**

1. **Claude Code's TodoWrite/TodoRead Tools** (Session-level task tracking)
   - Built-in Claude Code feature for managing tasks during a session
   - Lives in Claude Code's memory, not in any file
   - Cleared when session ends
   - Use for: Breaking down current work, tracking progress in real-time
   - Example: "Implement feature X" → subtasks → mark complete as you go

2. **TODO Section in WORKSPACE.md** (Project-level task tracking)
   - Part of your project's persistent memory across sessions
   - Lives in the WORKSPACE.md file (gitignored)
   - Persists between sessions
   - Use for: Tracking what needs to be done in the project
   - Example: "Implement missing operators", "Fix parser edge cases"

**Key Difference:** TodoWrite/TodoRead is for "what am I doing right now in this session", while WORKSPACE.md TODOs are for "what needs to be done in this project"

---

## 📂 WORKSPACE.md

### Overview
- **Purpose:** Store factual project state and context across Claude Code sessions
- **Philosophy:** A fact sheet, NOT a progress tracker or journal
- **Location:** `WORKSPACE.md` (gitignored)
- **Token Limit:** 800 tokens (~600 words)
- **Check Frequency:** Every 5-10 tool uses

### Creation Protocol (when missing)
When WORKSPACE.md doesn't exist, create it immediately with:
1. **Project identification** (name, version, current branch)
2. **Empty TODO section** ready for use
3. **Architecture Notes section** for discoveries
4. **Development Gotchas section** for important patterns
5. **Current Project State** placeholder

### Usage Protocol
1. **Session Start:** ALWAYS read WORKSPACE.md first
2. **During Work:** Update only when facts about the project change
3. **Token Check:** Run `wc -w WORKSPACE.md` regularly
4. **Over 600 words:** Prune immediately

### What to Track
**KEEP (High Value Facts):**
- Active TODOs (what needs doing, not what was done)
- Current blockers/issues
- Factual discoveries about codebase behavior
- Architecture patterns that exist in the code
- Development gotchas (facts about the system)

**REMOVE (Not Facts):**
- Completed tasks (including TODO items marked done)
- Progress narratives ("how we got here", "what we did")
- Solved problems (solutions are in the code)
- Session summaries or recaps
- "We discovered", "We implemented", "We fixed" statements
- Any form of progress tracking
- **Words like "implemented", "added", "updated", "extracted"** - these describe actions, not state
- **Temporal references** like "now", "recently", "after X"
- **Achievement language** like "successfully", "completed", "working"

### The Anti-Journal Mindset
**WORKSPACE.md is NOT a diary, changelog, or progress report.** It's a reference card for your next Claude Code session.

**Mental Exercise:** Imagine you're starting a fresh session tomorrow. WORKSPACE.md should tell you:
- What unusual patterns exist in the code
- What non-obvious design decisions were made
- What active work is pending
- What gotchas you might encounter

It should NOT tell you:
- What was done in previous sessions
- How the code evolved
- What problems were solved
- The implementation journey

### Anti-Violation Practices
**BEFORE adding anything new:**
1. **Remove something old first** - Practice "one in, one out"
2. **Never track progress** - Progress tracking belongs in git history, not WORKSPACE.md
3. **Challenge every line** - Ask "Is this a fact about the project?" If no, delete it
4. **State facts, not stories** - Write "X uses Y pattern" not "We discovered X uses Y"

### Formatting Rules
- Bullet points stating facts
- No progress indicators (✅, ❌, COMPLETED, etc.)
- No narrative words ("we", "discovered", "implemented")
- State facts: "X does Y" not "We made X do Y"
- Focus on "what is" not "what was done"
- **Verb choice matters:**
  - ✅ "has", "contains", "uses", "returns", "accepts"
  - ❌ "added", "implemented", "updated", "fixed", "extracted"

### Example: Facts vs Progress
**❌ WRONG (Progress Tracking):**
```
- Batch 15 Checkpoint 2.7: COMPLETED ✓
  - Added lifetime support to SelfKind enum
  - Updated extract_self_kind to capture lifetime from references
  - All tests passing
```

**✅ RIGHT (Stating Facts):**
```
- SelfKind enum has optional lifetime parameter
- extract_self_kind captures lifetimes from reference types
- convert_function handles functions, constants, and statics
- ItemKind derives from rustdoc item.kind field
```

### Pruning Priority
When over token limit, remove in order:
1. Any progress tracking or completion notes
2. Old session summaries or recaps
3. Historical narratives about problems or solutions
4. Duplicate information
5. Verbose explanations (state facts concisely)

---

## 💻 Development Workflow

### Code Quality Principles

#### Git History as Context
- **Documentation can lag behind implementation** - Task lists and plans may be out of date
- **Git commits tell the real story** - `git log --oneline` quickly shows what's been done
- **Common scenario:** Working on "pending" tasks that are already complete
- **Helpful when:** Resuming work, seeing "TODO" comments, or following batch plans

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
- **Focus on root causes** rather than symptoms
- **Verify your understanding** before proposing solutions

#### Prioritize Proper System Design Over Quick Fixes
- **Always implement the correct architecture first**
- **Resist shortcuts that create technical debt**
- **Design plugin-based architectures for extensibility**
- **Build proper abstraction layers between components**

#### Handling Systematic Issues - NO REFUSAL POLICY
- **When the user identifies hardcoded values or systematic issues:**
  - **DO NOT refuse to fix them properly** - Even if it requires significant changes
  - **DO NOT suggest band-aid solutions**
  - **DO investigate the root cause**
  - **DO implement the complete fix**

#### Fail-Fast Behavior
- **Prefer explicit panics over silent failures**
- **Use `panic!()` for "should never happen" scenarios**
- **Benefits:** Catches bugs early, makes assumptions explicit, prevents silent corruption

### Testing and Debugging

#### Test Failures - Question the Test Data First
**Problem:** Overengineering transpiler changes to make invalid test data pass  
**Solution:** When a test fails, first examine if the test data itself makes sense

**Analysis Protocol for Failing Tests:**
1. **Read the test name and assertion message** - What is it actually trying to test?
2. **Examine the test data** - Does it use valid/sensible examples?
3. **Check for nonsensical patterns:**
   - Importing from non-existent types (e.g., `MyType.myClone`)
   - Using undefined variables or functions
   - Syntax that violates language rules
4. **If test data is invalid:** Fix the test with valid examples that still test the intended behavior
5. **Only if test data is valid:** Then investigate transpiler changes

**Key Questions Before Changing Transpiler:**
- "Is this test data realistic/valid?"
- "What is this test actually trying to verify?"
- "Can I achieve the same test goal with valid data?"

#### Understanding Expected Output Files
**When user mentions problems with expected output files:**
1. **Read both files:** Look at the source `.vl` and its `.expected.rs` output
2. **Identify the issue:** What's missing, wrong, or incomplete in the output?
3. **Fix the transpiler:** The issue is almost always a transpiler bug that needs fixing
4. **Update expected file:** After fixing, update the expected output to match

**Remember:** Expected output files are test data showing current behavior. If the behavior looks wrong, fix the transpiler - don't just update the test file to hide the problem.

### Code Formatting

#### Rust Code Formatting
- **Tool:** Use `cargo fmt` to format all Rust files
- **When to run:** Before committing changes that include Rust files
- **Command:** `cargo fmt` (formats entire project)
- **Check formatting:** `cargo fmt --check` (verifies without modifying)

#### Avoid Redundant Builds
- **NEVER run `cargo build` unnecessarily** - other commands build automatically
- **`cargo test`** builds the project before running tests
- **`cargo run`** builds the project before running
- **`cargo fmt`** doesn't require a build

---

## 📝 File Management

### Critical Tool Limitation - Write/Edit Cannot Add Trailing Newlines
- **FACT:** The Write and Edit tools have a fundamental limitation - they CANNOT add trailing newlines
- **IMPACT:** Every file created or edited will lack the required trailing newline
- **MANDATORY:** After EVERY Write/Edit operation, you MUST manually add the trailing newline

### File Formatting Requirements
- **EVERY Git-tracked file MUST end with a trailing newline (empty line)**
  - Gitignored files do not require trailing newline checks
- All files MUST be formatted with the appropriate code formatter
- **Run `cargo fmt` before committing Rust code changes**

### Trailing Newline Enforcement - STRICT PROTOCOL

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
   If Git shows `\ No newline at end of file`, the file needs a newline.

3. **DO NOT TRUST these misleading sources:**
   - ❌ **Read tool visual output** - Cannot reliably show trailing newlines
   - ❌ **cargo fmt warnings** - Often wrong about newline status
   - ❌ **Visual inspection** - Humans cannot see trailing newlines

4. **STRICT RULES:**
   - **ALWAYS** assume Write/Edit left the file without a trailing newline
   - **ALWAYS** use `tail -c1 file | wc -l` to verify before adding
   - **NEVER** add a newline without verification (creates double newlines)
   - **NEVER** trust any source except `git diff --check`

**COMMON MISTAKES TO AVOID:**
- ❌ Adding newlines to files that already have them
- ❌ Running `echo "" >> filename` without checking first
- ❌ Assuming every edit removes the trailing newline

---

## 🔀 Git Workflow

### Staging and Committing - CRITICAL RULES
❌ **NEVER** run `git add` as a separate command  
✅ **ALWAYS** combine staging and committing in ONE tool call

**Standard Pattern:**
```bash
git add file1 file2 && git commit -m "message"
```

**When user explicitly requests "push":**
```bash
git add file1 file2 && git commit -m "message" && git push --no-progress
```

### Special Cases
- **File renames/moves:** Always include BOTH the old and new paths
  - **Best practice:** Use `git mv old_name new_name` - stages both automatically
  - **If using regular `mv`:** `git add old_name new_name && git commit -m "message"`
- **File deletions:** `git add deleted_file && git commit -m "message"`
- **Multiple operations:** Stage all affected files together

### Commit Process
1. **Analyze changes:** Run `git status`, `git diff`, and `git log` in parallel
2. **FORMAT CODE - ONLY IF RUST FILES CHANGED:** 
   - Check if any `.rs` files were modified: `git diff --name-only | grep -q '\.rs$'`
   - If Rust files changed: Run `cargo fmt`
3. **Verify formatting:** Check that new/edited files end with trailing newlines
4. **Stage + Commit (+ Push if requested):** Use combined commands

### Push Behavior
- **Push when explicitly requested** by user
- **Push after completing each significant task**
- Use `git push --no-progress` to suppress progress indicators

### Understanding Context When User Reports Git Actions
- **When user mentions they've done something with git:**
  - **Don't assume** - verify the actual repository state
  - **Check what happened** using git commands
  - **Understand the implications** before taking further git actions

### Creating Backup Branches
**EFFICIENT METHOD - Direct push to new remote branch:**
```bash
# Push current branch directly to a new remote branch without local creation
git push origin HEAD:backup-branch-name
```

### Interactive Rebase Limitations
❌ **NEVER use `git rebase -i`** - Interactive rebase requires user input which is not available

### Combining Commits - Basic Pattern
When user requests combining commits:

1. **Check current state and create backup**
2. **Hard reset to the base commit**
3. **Cherry-pick and amend the SPECIFIC commit**
4. **Cherry-pick any intermediate/subsequent commits**

**Important:** This rewrites history - requires force push if already pushed

---

## 🚀 Release Process

### Version Numbering Pattern
This project uses a development version with `-dev` suffix that gets released without the suffix:

1. **Development Version**: Work happens on version with `-dev` suffix (e.g., `0.1.2-dev`)
2. **Release Version**: When ready to release, remove `-dev` suffix (e.g., `0.1.2`)
3. **Next Development**: After tagging, bump to next version with `-dev` (e.g., `0.1.3-dev`)

### CHANGELOG.md Guidelines

**CRITICAL: Timing and Scope**
- Only include changes between the previous release tag and the current release commit
- Verify what existed in the previous release using git

**What to include:**
- User-facing changes to the transpiler functionality
- New language features or syntax support
- Bug fixes that affect transpiler output
- Breaking changes or deprecations

**What NOT to include:**
- Internal refactoring
- Changes to development documentation
- Test additions (unless they reveal fixed bugs)
- Code style/formatting changes

### Release Steps
1. **Update version for release** - Remove `-dev` suffix
2. **Update CHANGELOG.md**
3. **Commit the release**
4. **Create annotated tag** (without 'v' prefix)
5. **Push commits and tag**
6. **Begin next development cycle** - Add `-dev` suffix

---

## ⚠️ Common Pitfalls

### Session Management Issues

#### Skipping WORKSPACE.md on Session Start
**Problem:** Jumping directly into user requests without loading context  
**Solution:** Make reading WORKSPACE.md a reflex action  
**Self-Check:** "Have I read WORKSPACE.md yet?"

#### Not Maintaining TODO Section in WORKSPACE.md
**Problem:** Losing track of what needs to be done in the project across sessions  
**Solution:** Actively maintain TODO section in WORKSPACE.md (not Claude Code's TodoWrite/TodoRead)  
**Self-Check:** "Are the project's pending tasks documented in WORKSPACE.md?"

### WORKSPACE.md Management Issues

#### Not Capturing Important Insights
**Problem:** Missing valuable discoveries that would help future sessions  
**Solution:** When you find important patterns, document them  
**Self-Check:** "Did I learn something that would help future sessions?"

#### Failing to Update During Extended Sessions
**Problem:** Completing significant work without documenting discoveries  
**Solution:** Set systematic checkpoints every few tool uses  
**Triggers:** Any syntax error resolution, successful example creation, debugging breakthrough

### Technical Issues

#### Misinterpreting Trailing Newline Status
**Problem:** Adding unnecessary newlines or missing required newlines  
**Solution:** Follow the STRICT PROTOCOL in File Management section  
**Key Facts:**
- Write/Edit tools CANNOT add trailing newlines
- Only `git diff --check` is 100% reliable

#### Missing Files in Commits After Context Compaction
**Problem:** Multi-file operations often leave modified files uncommitted  
**Solution:** Before any commit, verify all work is complete  
**Common Patterns:**
- **Type changes:** type_checker.rs → parser.rs → codegen.rs → rust_interop.rs → tests → examples
- **Language features:** ast.rs → parser.rs → codegen.rs → examples → tests

### Mindset Issues

#### Taking Responsibility When Breaking Things
**Problem:** Blaming "fundamental issues" when your changes cause test failures  
**Solution:** Take responsibility for breaking changes  
**Correct Response:**
- Acknowledge the mistake immediately
- Understand what went wrong before trying again
- Don't blame the architecture

---

## 📚 Project-Specific Notes

### Important Instructions
Codebase and user instructions are shown below. Be sure to adhere to these instructions. IMPORTANT: These instructions OVERRIDE any default behavior and you MUST follow them exactly as written.

### File Footer Requirements
**EVERY Git-tracked file MUST end with a trailing newline (empty line)**
