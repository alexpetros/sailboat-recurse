import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { SAILBOAT_URL } from './setup.js'

describe('Hello world test', async () => {

  it('shows "Hello World!"', async () => {
    const res = await fetch(`${SAILBOAT_URL}`)
    const body = await res.text()
    assert(body.includes("<h1>Alex's Sailboat!</h1>"))
    assert(body.includes("<h2>Feeds</h2>"))
  })

})
