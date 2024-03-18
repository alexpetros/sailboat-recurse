import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { SAILBOAT_URL } from './setup.js'

describe('/.well_known/webfinger', async () => {

  it('Responds with okay', async () => {
    const res = await fetch(`${SAILBOAT_URL}/.well_known/webfinger`)
    const body = await res.text()

    assert.equal(res.status, 200)
    assert.equal(body, "")
  })

})

