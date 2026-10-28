import assert from 'node:assert/strict'
import { test } from 'node:test'
import { episodeCode, fillDaily } from './stats.ts'

const DAY_MS = 86_400_000

test('fillDaily pads missing days with zeros, oldest first', () => {
  const now = 20_000 * DAY_MS + 3_600_000
  const out = fillDaily(
    [
      { day: 20_000, seconds: 90 },
      { day: 19_998, seconds: 30 },
      { day: 19_000, seconds: 5 },
    ],
    5,
    now,
  )
  assert.deepEqual(
    out.map((d) => d.day),
    [19_996, 19_997, 19_998, 19_999, 20_000],
  )
  assert.deepEqual(
    out.map((d) => d.seconds),
    [0, 0, 30, 0, 90],
  )
})

test('fillDaily with no data is all zeros', () => {
  const out = fillDaily([], 30, 0)
  assert.equal(out.length, 30)
  assert.ok(out.every((d) => d.seconds === 0))
})

test('episodeCode pads', () => {
  assert.equal(episodeCode(1, 2), 'S01E02')
  assert.equal(episodeCode(12, 103), 'S12E103')
  assert.equal(episodeCode(null, null), 'S00E00')
})
