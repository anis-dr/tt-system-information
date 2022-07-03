import test from 'ava'

import sys from '../index.js'

test('load average returns a result', (t) => {
    t.truthy(sys.loadAverageInfo())
})
