import { describe, expect, it } from 'vitest'

import { resolveApiBaseUrl } from '../api'

describe('resolveApiBaseUrl', () => {
  it('prefers an explicit API URL when provided', () => {
    expect(
      resolveApiBaseUrl(
        'http://192.168.1.20:8790',
        '8790',
        { origin: 'http://192.168.1.10:4178', protocol: 'http:', hostname: '192.168.1.10' },
      ),
    ).toBe('http://192.168.1.20:8790')
  })

  it('uses same-origin when no API URL or API port is configured', () => {
    expect(
      resolveApiBaseUrl(
        '',
        '',
        { origin: 'http://192.168.1.10:8790', protocol: 'http:', hostname: '192.168.1.10' },
      ),
    ).toBe('http://192.168.1.10:8790')
  })

  it('derives the API host from the current page hostname when an API port is provided', () => {
    expect(
      resolveApiBaseUrl(
        '',
        '8794',
        { origin: 'http://192.168.1.10:4178', protocol: 'http:', hostname: '192.168.1.10' },
      ),
    ).toBe('http://192.168.1.10:8794')
  })

  it('falls back to localhost when no location is available', () => {
    expect(resolveApiBaseUrl(undefined, '8790', null)).toBe('http://127.0.0.1:8790')
  })
})
