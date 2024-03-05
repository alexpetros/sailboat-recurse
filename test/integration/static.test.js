import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { SAILBOAT_URL } from './setup.js'

describe('Static files test', async () => {

  it('finds common css', async () => {
    const res = await fetch(`${SAILBOAT_URL}/static/common.css`)
    const body = await res.text()
    assert.match(body, /nav {/)
    assert.equal(res.status, 200)
  })

  it('finds hello world js', async () => {
    const res = await fetch(`${SAILBOAT_URL}/static/hello.js`)
    const body = await res.text()
    assert.equal(body, "console.log('Hello, World!')\n")
    assert.equal(res.status, 200)
  })

  it('returns a 404 on an unknown file', async () => {
    const res = await fetch(`${SAILBOAT_URL}/static/unknown.js`)
    assert.equal(res.status, 404)
  })

})


