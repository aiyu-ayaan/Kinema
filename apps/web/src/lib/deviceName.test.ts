import assert from 'node:assert/strict'
import { test } from 'node:test'
import { describeDevice } from './deviceName.ts'

const chromeWin = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'

test('names the browser and the system', () => {
  assert.equal(describeDevice(chromeWin, false, ''), 'Chrome on Windows')
  assert.equal(describeDevice(`${chromeWin} Edg/120.0`, false, ''), 'Edge on Windows')
  assert.equal(describeDevice('Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0', false, ''), 'Firefox on Linux')
  assert.equal(describeDevice('Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1', false, ''), 'Safari on iOS')
  assert.equal(describeDevice('Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/120.0 Mobile Safari/537.36', false, ''), 'Chrome on Android')
})

test('desktop app uses the platform instead of the user agent', () => {
  assert.equal(describeDevice(chromeWin, true, 'linux'), 'Kinema desktop (linux)')
  assert.equal(describeDevice(chromeWin, true, ''), 'Kinema desktop')
})

test('unknown agents still get a name', () => {
  assert.equal(describeDevice('', false, ''), 'Browser')
})
