const { test } = require('node:test')
const assert = require('node:assert/strict')
const { channelOf, pick, suffixFor } = require('../src/updates')

const release = (tag, ...names) => ({
  tag_name: `v${tag}`,
  html_url: `p/${tag}`,
  assets: names.map((name) => ({ name, browser_download_url: `u/${name}` })),
})
const releases = [
  release('0.0.2-alpha.1', 'Kinema-0.0.2-alpha.1.AppImage'),
  release('0.0.1', 'Kinema-0.0.1.AppImage', 'Kinema-Desktop-0.0.1-Setup.exe'),
  release('0.0.1-beta.2', 'Kinema-0.0.1-beta.2.AppImage'),
  release('0.0.3', 'Kinema-0.0.3.apk'),
]

test('an alpha install takes the newest alpha', () => {
  assert.deepEqual(pick(releases, '0.0.1-alpha.1', 'alpha', '.AppImage'), {
    version: '0.0.2-alpha.1',
    url: 'u/Kinema-0.0.2-alpha.1.AppImage',
    page: 'p/0.0.2-alpha.1',
  })
})

test('beta takes stable too, never alphas, and skips releases without its asset', () => {
  assert.equal(pick(releases, '0.0.1-beta.2', 'beta', '.AppImage').version, '0.0.1')
  assert.equal(pick(releases, '0.0.1-beta.2', 'beta', '-Setup.exe').version, '0.0.1')
  assert.equal(pick(releases, '0.0.1', 'stable', '.AppImage'), null)
  assert.equal(pick(releases, 'not-a-version', 'alpha', '.AppImage'), null)
})

test('channel and asset follow the install', () => {
  assert.equal(channelOf('0.0.1-alpha.1'), 'alpha')
  assert.equal(channelOf('1.2.3'), 'stable')
  assert.equal(suffixFor('linux', '/home/a/Kinema.AppImage'), '.AppImage')
  assert.equal(suffixFor('win32', undefined), '-Setup.exe')
  assert.equal(suffixFor('linux', undefined), '.deb')
})
