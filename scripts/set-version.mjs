#!/usr/bin/env node
// Writes one version into every manifest, so the server, the web UI, the desktop app and the
// Android app (which reads the root package.json) all carry the release's name.
//   node scripts/set-version.mjs 0.0.1-alpha.1 [--bump-build]
// --bump-build also raises `build` in the root package.json: Android's versionCode and the
// build number the web UI shows.
import { readFileSync, writeFileSync } from 'node:fs'

const [version, flag] = process.argv.slice(2)
if (!/^\d+\.\d+\.\d+(-(alpha|beta)\.\d+)?$/.test(version ?? '')) throw new Error(`Invalid version '${version}'.`)

const edit = (path, change) => writeFileSync(path, change(readFileSync(path, 'utf8')))
const json = (path, change) => edit(path, (text) => JSON.stringify(change(JSON.parse(text)), null, 2) + '\n')
// Fails loudly when a manifest no longer has the line, rather than releasing a stale version.
const replace = (text, pattern, value, path) => {
  if (!pattern.test(text)) throw new Error(`${path}: nothing matches ${pattern}`)
  return text.replace(pattern, value)
}

json('package.json', (p) => ({ ...p, version, build: (p.build ?? 1) + (flag === '--bump-build' ? 1 : 0) }))
for (const path of ['apps/web/package.json', 'apps/desktop/package.json']) json(path, (p) => ({ ...p, version }))
edit('Cargo.toml', (t) => replace(t, /(\[workspace\.package\]\nversion = ")[^"]*/, `$1${version}`, 'Cargo.toml'))
for (const crate of ['backend', 'kinema-core', 'kinema-db', 'kinema-media', 'kinema-meta', 'kinema-scanner']) {
  edit('Cargo.lock', (t) => replace(t, new RegExp(`(name = "${crate}"\\nversion = ")[^"]*`), `$1${version}`, 'Cargo.lock'))
}
console.log(`version ${version}, build ${JSON.parse(readFileSync('package.json', 'utf8')).build}`)
