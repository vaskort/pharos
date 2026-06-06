import { type MetadataRoute } from 'next'

const siteUrl = 'https://pharos-cli.io'

export const dynamic = 'force-static'

const routes = [
  '',
  '/docs/understanding-output',
  '/docs/ci-json',
  '/docs/cli',
  '/docs/lockfiles',
  '/docs/limitations',
]

export default function sitemap(): MetadataRoute.Sitemap {
  return routes.map((route) => ({
    url: `${siteUrl}${route}/`,
    changeFrequency: 'weekly',
    priority: route === '' ? 1 : 0.7,
  }))
}
