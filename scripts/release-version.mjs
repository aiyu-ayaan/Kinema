#!/usr/bin/env node
// Works out the next version from the commit subjects of a push to master.
// A release is asked for by a marker at the START of a commit subject:
//
//   !alpha   0.0.0         -> 0.0.1-alpha.1  pre-release (0.0.0 = nothing released yet)
//            0.0.1-alpha.1 -> 0.0.1-alpha.2
//   !beta    0.0.1-alpha.3 -> 0.0.1-beta.1   promote, same base
//   !stable  0.0.1-beta.2  -> 0.0.1          promote to stable, no bump
//   !fix     1.4.2         -> 1.4.3          stable
//   !feat    1.4.2         -> 1.5.0          stable
//   !major   1.4.2         -> 2.0.0          stable
//   !patch   0.0.2         -> 0.0.2          no bump: rebuild that version in place
//
// A marker may name the platforms it builds, in the scope position:
// `!alpha(android)`, `!fix(docker,desktop)`. A scope with any other word in it is
// an ordinary commit scope and builds everything. When several pushed commits
// carry markers the strongest wins, in the order of MARKERS.
// Full rules: development/docs/RELEASE.md.
//
//   node scripts/release-version.mjs --current 0.0.0 --subject "!alpha : first"
//   node scripts/release-version.mjs --check      # self-check

import { strictEqual, throws } from 'node:assert'
import { readFileSync } from 'node:fs'

const CHANNELS = { alpha: 1, beta: 2 }
const MARKERS = ['major', 'feat', 'fix', 'stable', 'beta', 'alpha', 'patch']
const VERSION_RE = /^(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta)\.(\d+))?$/
const MARKER_RE = /^\s*!(major|feat|fix|stable|alpha|beta|patch)(?![a-z0-9])(?:\(([^)]*)\))?/i

export const TARGETS = ['server', 'docker', 'desktop', 'android']
const ALIASES = {
  server: 'server', tarball: 'server', bundle: 'server',
  docker: 'docker', image: 'docker',
  desktop: 'desktop', linux: 'desktop', electron: 'desktop',
  android: 'android', apk: 'android', mobile: 'android',
}

export function parseVersion(version) {
  const m = VERSION_RE.exec(String(version).trim())
  if (!m) throw new Error(`Invalid version '${version}'. Expected X.Y.Z or X.Y.Z-alpha.N.`)
  return { major: +m[1], minor: +m[2], patch: +m[3], channel: m[4] ?? '', pre: m[5] ? +m[5] : 0 }
}

// The release flow's own commits (chore(release): vX) never start with `!`, so they are not markers.
export function detectMarker(subjects) {
  const found = new Set(subjects.map((s) => MARKER_RE.exec(s)?.[1].toLowerCase()))
  return MARKERS.find((marker) => found.has(marker)) ?? ''
}

export function parseTargets(subjects) {
  const chosen = new Set()
  for (const subject of subjects) {
    const names = (MARKER_RE.exec(subject)?.[2] ?? '').split(/[,+\s]+/).map((n) => n.toLowerCase()).filter(Boolean)
    // Only a scope made entirely of platform names is a platform list.
    if (!names.length || !names.every((n) => n === 'all' || n in ALIASES)) continue
    for (const name of names) (name === 'all' ? TARGETS : [ALIASES[name]]).forEach((t) => chosen.add(t))
  }
  return chosen.size ? TARGETS.filter((t) => chosen.has(t)) : [...TARGETS]
}

export function nextVersion(current, marker) {
  const { major, minor, patch, channel, pre } = parseVersion(current)
  const base = `${major}.${minor}.${patch}`
  switch (marker) {
    case 'major': return `${major + 1}.0.0`
    case 'feat': return `${major}.${minor + 1}.0`
    case 'fix': return `${major}.${minor}.${patch + 1}`
    case 'stable':
      if (!channel) throw new Error(`!stable needs a pre-release to promote; ${current} is already stable.`)
      return base
    case 'patch': return String(current).trim()
    case 'alpha':
    case 'beta':
      // Same or lower channel: count up or promote on the same base. Otherwise
      // (from stable, or !alpha on a beta) start on the next patch.
      if (channel && CHANNELS[channel] <= CHANNELS[marker]) return `${base}-${marker}.${channel === marker ? pre + 1 : 1}`
      return `${major}.${minor}.${patch + 1}-${marker}.1`
    default: throw new Error(`Unknown marker '${marker}'.`)
  }
}

function selfCheck() {
  const eq = (current, marker, expected) => strictEqual(nextVersion(current, marker), expected, `${current} + !${marker}`)
  // The first release of a repository with no tag.
  eq('0.0.0', 'alpha', '0.0.1-alpha.1')
  eq('0.0.0', 'beta', '0.0.1-beta.1')
  eq('0.0.0', 'fix', '0.0.1')
  eq('0.0.1-alpha.1', 'alpha', '0.0.1-alpha.2')
  eq('0.0.1-alpha.3', 'beta', '0.0.1-beta.1')
  eq('0.0.1-beta.1', 'beta', '0.0.1-beta.2')
  eq('0.0.1-beta.2', 'stable', '0.0.1')
  eq('0.0.1-beta.2', 'alpha', '0.0.2-alpha.1')
  eq('0.0.1', 'alpha', '0.0.2-alpha.1')
  eq('0.0.1', 'feat', '0.1.0')
  eq('0.1.0', 'major', '1.0.0')
  eq('0.0.2-beta.1', 'fix', '0.0.3')
  eq('0.0.2-alpha.3', 'patch', '0.0.2-alpha.3')
  throws(() => nextVersion('0.0.1', 'stable'), /already stable/)
  throws(() => parseVersion('1.2.3-rc.1'), /Invalid version/)

  strictEqual(detectMarker(['fix: ordinary commit', '!feat: new thing']), 'feat')
  strictEqual(detectMarker(['!fix: a', '!major: b']), 'major')
  strictEqual(detectMarker(['no marker here']), '')

  strictEqual(parseTargets(['!alpha: all']).join(','), TARGETS.join(','))
  strictEqual(parseTargets(['!alpha(android): only apk']).join(','), 'android')
  strictEqual(parseTargets(['!fix(docker,desktop): two']).join(','), 'docker,desktop')
  strictEqual(parseTargets(['!fix(server): server tarball']).join(','), 'server')
  strictEqual(parseTargets(['!feat(auth): not a platform scope']).join(','), TARGETS.join(','))
  console.log('release-version: ok')
}

function parseArgs(args) {
  const opts = { current: '', subjects: [] }
  for (let i = 0; i < args.length; i++) {
    const a = args[i]
    if (a === '--check') return { check: true }
    if (a === '--current') opts.current = args[++i]
    else if (a === '--subject') opts.subjects.push(args[++i])
    else if (a === '--subjects-file') {
      opts.subjects.push(...readFileSync(args[++i], 'utf8').split('\n').filter(Boolean))
    }
  }
  return opts
}

const opts = parseArgs(process.argv.slice(2))
if (opts.check) {
  selfCheck()
  process.exit(0)
}

if (!opts.current) {
  console.error('Usage: release-version.mjs --current <v> (--subject <s> | --subjects-file <f>)')
  process.exit(1)
}

const marker = detectMarker(opts.subjects)
if (!marker) {
  console.log('release=false')
  process.exit(0)
}

const version = nextVersion(opts.current, marker)
const targets = parseTargets(opts.subjects)
console.log(`release=true\nmarker=${marker}\nversion=${version}\ntargets=${targets.join(',')}`)
