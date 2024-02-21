import { describe, it } from 'node:test'
import assert from 'node:assert/strict'

import { SAILBOAT_URL } from './setup.js'
console.log(SAILBOAT_URL)

describe('echo tests', async () => {

  it('says hello world', async () => {
    const res = await fetch(SAILBOAT_URL)
    const body = await res.text()
    assert.equal(body, 'Hello, World!\n')
  })

  it('echoes a post body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch(`${SAILBOAT_URL}/echo`, { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '{"test":"hi"}')
  })

  it('echoes an uppercased post body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch(`${SAILBOAT_URL}/echo/uppercase`, { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '{"TEST":"HI"}')
  })

  it('echoes a reversed body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch(`${SAILBOAT_URL}/echo/reversed`, { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '}"ih":"tset"{')
  })

})
