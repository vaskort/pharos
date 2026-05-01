import { fileURLToPath } from 'node:url'
import { dirname } from 'node:path'

import withMarkdoc from '@markdoc/next.js'

import withSearch from './src/markdoc/search.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))

/** @type {import('next').NextConfig} */
const nextConfig = {
  pageExtensions: ['js', 'jsx', 'md', 'ts', 'tsx'],
  output: 'export',
  basePath: '/pharos',
  trailingSlash: true,
  images: { unoptimized: true },
  outputFileTracingRoot: __dirname,
}

export default withSearch(
  withMarkdoc({ schemaPath: './src/markdoc', nextjsExports: ['revalidate'] })(
    nextConfig,
  ),
)
