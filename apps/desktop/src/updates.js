// Which GitHub release this copy should update to. Pure, so tests/updates.test.js can check it.
// The release workflow names versions 0.0.2-alpha.1, 0.0.2-beta.1 and 0.0.2 (scripts/release-version.mjs).

const RELEASES = 'https://api.github.com/repos/aiyu-ayaan/Kinema/releases?per_page=30'
const VERSION = /^v?(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta)\.(\d+))?$/
const STAGE = { alpha: 0, beta: 1, stable: 2 }

/** `1.2.3-beta.4` as [major, minor, patch, stage, number]; null for any other shape. */
function versionKey(text) {
  const m = VERSION.exec(String(text).trim())
  // A stable release has no pre-release number and still sorts above every one of its pre-releases.
  return m && [+m[1], +m[2], +m[3], STAGE[m[4] ?? 'stable'], m[5] ? +m[5] : Infinity]
}
const newer = (a, b) => {
  const i = a.findIndex((part, n) => part !== b[n])
  return i !== -1 && a[i] > b[i]
}

/** The channel a build belongs to: whoever installed an alpha wants alphas. */
const channelOf = (version) => Object.keys(STAGE).find((name) => STAGE[name] === versionKey(version)?.[3]) ?? 'stable'

/**
 * The newest release on `channel` (which also takes every steadier one) that is newer than
 * `installed` and has an asset ending in `suffix`. Releases without one are passed over.
 */
function pick(releases, installed, channel, suffix) {
  const current = versionKey(installed)
  let best = null
  for (const release of current ? releases : []) {
    const key = versionKey(release.tag_name)
    const asset = (release.assets ?? []).find((a) => a.name.endsWith(suffix))
    if (release.draft || !key || !asset || key[3] < STAGE[channel] || !newer(key, best?.key ?? current)) continue
    best = { key, version: release.tag_name.replace(/^v/, ''), url: asset.browser_download_url, page: release.html_url }
  }
  return best && { version: best.version, url: best.url, page: best.page }
}

/** The asset this install can apply: its own AppImage, else the installer for the platform. */
const suffixFor = (platform, appImage) => (appImage ? '.AppImage' : platform === 'win32' ? '-Setup.exe' : '.deb')

module.exports = { RELEASES, STAGE, channelOf, pick, suffixFor }
