export const meta = {
  name: 'release-changelog',
  description: 'Investigate changes since the last release and write a new CHANGELOG.md release section',
  whenToUse: 'Cutting a release. Pass args {version, date?, lastTag?} — investigates every commit since the last tag and edits CHANGELOG.md in place.',
  phases: [
    { title: 'Survey', detail: 'collect commits since the last release tag' },
    { title: 'Investigate', detail: 'one agent per meaningful change → structured entry' },
    { title: 'Write', detail: 'synthesize entries and edit CHANGELOG.md' },
  ],
}

// ---- release knobs ----------------------------------------------------
// The `args` global is not delivered to scripts in this runtime, so the
// release is parameterized by these consts. For a normal release leave them
// as-is: the next version is auto-derived by bumping the requested segment of
// the previous tag. For an arbitrary release, set VERSION_OVERRIDE.
const VERSION_OVERRIDE = null   // e.g. '0.1.0' to force an exact version; null = auto-bump
const BUMP = 'patch'            // 'patch' | 'minor' | 'major' — segment to increment when auto-bumping
const LAST_TAG_OVERRIDE = null  // e.g. 'v0.0.6'; null = `git describe --tags --abbrev=0`
const lastTagHint = LAST_TAG_OVERRIDE

function bump(tag, which) {
  const m = String(tag).match(/(\d+)\.(\d+)\.(\d+)/)
  if (!m) throw new Error(`cannot parse a semver from last tag "${tag}"`)
  let [maj, min, pat] = [Number(m[1]), Number(m[2]), Number(m[3])]
  if (which === 'major') { maj += 1; min = 0; pat = 0 }
  else if (which === 'minor') { min += 1; pat = 0 }
  else { pat += 1 }
  return `${maj}.${min}.${pat}`
}

// ---- schemas ----------------------------------------------------------
const SURVEY_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['lastTag', 'repoUrl', 'today', 'commits'],
  properties: {
    lastTag: { type: 'string', description: 'The previous release tag, e.g. v0.0.6' },
    repoUrl: { type: 'string', description: 'Canonical GitHub web URL, e.g. https://github.com/harmont-dev/harmont-cli' },
    today: { type: 'string', description: "Today's date as YYYY-MM-DD" },
    commits: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['hash', 'author', 'subject', 'prNumber', 'meaningful'],
        properties: {
          hash: { type: 'string' },
          author: { type: 'string' },
          subject: { type: 'string' },
          prNumber: { type: ['integer', 'null'] },
          meaningful: { type: 'boolean', description: 'false for chore/CI/version-bump/merge noise that does not belong in a changelog' },
        },
      },
    },
  },
}

const ENTRY_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['skip', 'entries'],
  properties: {
    skip: { type: 'boolean', description: 'true if this commit is noise and should not appear in the changelog' },
    entries: {
      type: 'array',
      description: 'One or more changelog bullets derived from this change (a PR may touch multiple categories)',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['category', 'area', 'breaking', 'text', 'refLabel', 'refUrl', 'externalAuthor'],
        properties: {
          category: { type: 'string', enum: ['Changed', 'Added', 'Removed', 'Fixed'] },
          area: { type: ['string', 'null'], enum: ['CLI', 'DSL', 'SDK', null], description: 'Bold prefix; null if none fits' },
          breaking: { type: 'boolean' },
          text: { type: 'string', description: 'Bullet body only — no leading dash, no bold prefix, no ref link, no author' },
          refLabel: { type: 'string', description: 'e.g. "#140" for a PR, or a 7-char commit hash' },
          refUrl: { type: 'string', description: 'Full URL the ref points to (PR or commit)' },
          externalAuthor: { type: ['string', 'null'], description: 'Display name in trailing parens, ONLY for non-maintainer contributors; null otherwise' },
        },
      },
    },
  },
}

const WRITE_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['sectionMarkdown', 'refsAdded'],
  properties: {
    sectionMarkdown: { type: 'string', description: 'The full new release section as written into the file' },
    refsAdded: { type: 'array', items: { type: 'string' } },
  },
}

// ---- Survey -----------------------------------------------------------
phase('Survey')
const survey = await agent(
  `You are surveying git history to prepare a CHANGELOG release.\n\n` +
  `Run these in the repo root:\n` +
  `1. Determine the previous release tag. ${lastTagHint ? `Use "${lastTagHint}".` : 'Run: git describe --tags --abbrev=0'}\n` +
  `2. git log <lastTag>..HEAD --format='%h | %an | %s'\n` +
  `3. git remote get-url origin  (normalize ssh/scp form to an https web URL, strip trailing .git)\n` +
  `4. date +%Y-%m-%d  → return as "today"\n\n` +
  `For each commit return hash, author, subject, prNumber (parse a trailing "(#NNN)" from the subject; null if none), ` +
  `and meaningful=false for changelog noise: "run ci", "auto-versioned ...", "bump version", merge commits, pure formatting/whitespace. ` +
  `Everything user- or developer-facing is meaningful=true.`,
  { schema: SURVEY_SCHEMA, label: 'survey', phase: 'Survey' }
)

const repoUrl = survey.repoUrl.replace(/\.git$/, '')
const version = VERSION_OVERRIDE || bump(survey.lastTag, BUMP)
const date = survey.today
const meaningful = survey.commits.filter((c) => c.meaningful)
log(`${survey.lastTag} → ${version} (${date}): ${survey.commits.length} commits, ${meaningful.length} meaningful`)

// ---- Investigate ------------------------------------------------------
phase('Investigate')
const investigated = await parallel(
  meaningful.map((c) => () =>
    agent(
      `Investigate one change and produce changelog bullet(s) in Keep-a-Changelog style.\n\n` +
      `Commit: ${c.hash}\nSubject: ${c.subject}\nAuthor: ${c.author}\nPR: ${c.prNumber ? '#' + c.prNumber : 'none'}\n\n` +
      `Read the actual change:\n` +
      `- git show --stat ${c.hash}\n` +
      (c.prNumber ? `- gh pr view ${c.prNumber} --json title,author,body  (use the PR body for the WHY)\n` : '') +
      `\nClassify into category (Changed/Added/Removed/Fixed), area (CLI/DSL/SDK or null), and breaking (bool). ` +
      `Write each bullet as a tight, user-facing sentence describing the OUTCOME, not the implementation. Past where it helps, name the new command/flag/API. ` +
      `Split into multiple entries only when a change genuinely spans categories (e.g. a feature that also removes dead behavior).\n\n` +
      `refLabel/refUrl: prefer the PR (${c.prNumber ? `#${c.prNumber} → ${repoUrl}/pull/${c.prNumber}` : `none — use commit ${c.hash} → ${repoUrl}/commit/${c.hash}`}).\n` +
      `externalAuthor: the contributor's display name ONLY if they are NOT the repo maintainer (maintainer = the dominant committer); else null.\n` +
      `If on reflection this change is pure noise, set skip=true with an empty entries array.`,
      { schema: ENTRY_SCHEMA, label: c.prNumber ? `#${c.prNumber}` : c.hash, phase: 'Investigate' }
    )
  )
)

const allEntries = investigated
  .filter(Boolean)
  .filter((r) => !r.skip)
  .flatMap((r) => r.entries)

if (allEntries.length === 0) {
  log('No meaningful changes found — nothing to release.')
  return { version, lastTag: survey.lastTag, entries: [], note: 'empty' }
}

// ---- Write ------------------------------------------------------------
phase('Write')
const ORDER = ['Changed', 'Added', 'Removed', 'Fixed']
const grouped = ORDER.map((cat) => ({ cat, items: allEntries.filter((e) => e.category === cat) })).filter((g) => g.items.length)
const entriesJson = JSON.stringify(grouped, null, 2)

const write = await agent(
  `Edit CHANGELOG.md to cut release ${version}${date ? ` dated ${date}` : ''}. Match the file's EXISTING formatting exactly — read it first.\n\n` +
  `Entries to write, already grouped and ordered (Changed, Added, Removed, Fixed):\n${entriesJson}\n\n` +
  `Repo web URL: ${repoUrl}\n\n` +
  `Rules, derived from the existing entries in the file:\n` +
  `- Each bullet: "- " then, if breaking, "**Breaking:** ", then if area set "**<AREA>:** ", then the text, then " (${'[refLabel][ref<n>]'} link)" as "([${'<refLabel>'}][${'<linkid>'}])", then if externalAuthor " (<name>)".\n` +
  `  Concretely a PR ref "#140" renders inline as "([#140][pr140])" and needs a link def "[pr140]: ${repoUrl}/pull/140". A commit ref like "1bf727e" renders "([\`1bf727e\`][c1bf727e])" with def "[c1bf727e]: ${repoUrl}/commit/1bf727e".\n` +
  `- Insert a new "## [${version}] - ${date || '<DATE>'}" section immediately BELOW the "## [Unreleased]" heading, leaving "## [Unreleased]" present and empty.\n` +
  `- Under it emit each non-empty category as "### <Category>" with its bullets, in the given order.\n` +
  `- Append the new link-reference definitions to the link-def block at the BOTTOM of the file, next to the existing ones. Do not duplicate a def that already exists.\n` +
  `- Do NOT touch existing released sections.\n\n` +
  `Apply the edit with the Edit tool, then return the new section markdown and the list of link defs you added.`,
  { schema: WRITE_SCHEMA, label: 'write-changelog', phase: 'Write' }
)

return {
  version,
  date,
  lastTag: survey.lastTag,
  entryCount: allEntries.length,
  section: write.sectionMarkdown,
  refsAdded: write.refsAdded,
}
